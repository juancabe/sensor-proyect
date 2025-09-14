use crate::{
    ble_protocol::BleInitialConfigImplementation,
    client_state::ClientState,
    helpers::get_random_buf,
    persistence::Persistence,
    sensors::{Scd41InitData, Scd41WorkingMode, Sensors},
    server_communicator::ServerCommunicator,
    wifi_connector::{WifiClientConfig, WifiConnector},
};
use common::types::{validate::device_id::DeviceId, zstr20::ZStr20};
use common::{
    auth::{self, keys::Keys},
    endpoints_io::sensor_data::PostSensorData,
};
use esp_idf_svc::{
    eventloop::EspEventLoop,
    hal::{i2c::I2cDriver, prelude::*},
    nvs::EspDefaultNvsPartition,
    wifi::Configuration,
};
use esp_idf_sys::{
    esp_get_free_heap_size, esp_get_minimum_free_heap_size, esp_read_mac,
    uxTaskGetStackHighWaterMark,
};

pub mod ble_protocol;
pub mod client_state;
pub mod helpers;
pub mod persistence;
pub mod private;
pub mod sensors;
pub mod server_communicator;
pub mod wifi_connector;

const GENERATE_COMMUNICATOR_RETRIES: usize = 10;
const POST_DATA_RETRIES: usize = 10;
const MAX_CONSECUTIVE_MEASUREMENT_ERRORS_ALLOWED: usize = 10;
const MAX_SEND_DATA_LOOP_FAILED_ON_POST_DATA_ERRORS_ALLOWED: usize = 10;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let mut state = ClientState::new();

    let p = Peripherals::take().unwrap();
    let sysloop = EspEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().expect("Couldn't take NVS Default Partition");
    let persistence = Persistence::new(nvs.clone()).expect("Persistence shpould be available");

    let device_id = get_device_id();
    log::info!("device_id: {device_id:?}");

    // --- Fetch persisted data
    let wifi_buf = &mut [0u8; persistence::Keys::WifiConfigSerialized.min_recv_buffer_size()];
    let wifi = if persistence
        .get(persistence::Keys::WifiConfigSerialized, wifi_buf)
        .expect("Wifi config should be recoverable")
    {
        let built_str = str::from_utf8(wifi_buf)
            .expect("Wifi buf should be valid utf8")
            .split_terminator("\0")
            .next()
            .expect("Should contain something");
        let wifi: WifiClientConfig = serde_json::from_str(built_str)
            .expect("wifi_string should be valid WifiClientConfig serialized structure");
        Some(wifi)
    } else {
        None
    };

    let auth_keys_buf = &mut [0u8; persistence::Keys::AuthKeysSerialized.min_recv_buffer_size()];
    let auth_keys = if persistence
        .get(persistence::Keys::AuthKeysSerialized, auth_keys_buf)
        .expect("auth_keys config should be recoverable")
    {
        let built_str = str::from_utf8(&auth_keys_buf[..auth_keys_buf.len() - 1])
            .expect("auth_keys buf should be valid utf8")
            .split_terminator("\0")
            .next()
            .expect("Should contain something");
        let auth_keys: auth::keys::Keys = serde_json::from_str(built_str)
            .expect("auth_keys_string should be valid Keys serialized structure");
        Some(auth_keys)
    } else {
        None
    };
    // ---

    // --- Decide what to do with the persisted data
    let state_persistence = state.apply_persistence(wifi, auth_keys);
    let (_wifi, mut keys) = match state.get_state() {
        client_state::State::UnsetPersistence | client_state::State::InconsistentPersistence => {
            initial_config(device_id, persistence)
        }
        client_state::State::PersistenceGot => {
            let state_persistence = state_persistence.expect("Should be Some");
            let wifi = WifiConnector::from_config(
                2,
                &Configuration::Client(state_persistence.wifi.into()),
                p.modem,
                sysloop,
                nvs.clone(),
            )
            .expect("Should connect to Wifi");

            (wifi, state_persistence.auth_keys)
        }
        _ => panic!("Unexpected state: {state:?}", state = state.get_state()),
    };
    // ---

    let i2c_config = esp_idf_svc::hal::i2c::I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(p.i2c0, p.pins.gpio4, p.pins.gpio3, &i2c_config)
        .expect("Should be able to create I2CDriver");
    let mut sensors = match Sensors::new(
        i2c,
        Some(()),
        Some(Scd41InitData::new(Scd41WorkingMode::LowPower)),
        false,
    ) {
        Ok(s) => s,
        Err((_i2c, scd41, aht10)) => {
            panic!(
                "Unable to create sensor driver\nscd41: {:?}\naht10: {:?}\n\n",
                scd41, aht10
            );
        }
    };

    let mut communicator = None;
    for _ in 0..GENERATE_COMMUNICATOR_RETRIES {
        let comm = ServerCommunicator::generate(&mut keys, device_id.clone());
        match comm {
            Ok(c) => {
                log::info!("ServerCommunicator created");
                communicator = Some(c);
                break;
            }
            Err(e) => match e {
                server_communicator::Error::UnexpectedResponse(code) => match code {
                401 // UNAUTHORIZED
                | 404 // NOT FOUND
                    => {
                        handle_unauthorized(persistence);
                }
                c => {
                    log::warn!("Got unexpected response from server: {c}");
                }
            },
                error => {
                    log::warn!("Error occured: {error:?}")
                }
            },
        }
        log::info!("Retrying");
    }

    let mut communicator = communicator.expect("GENERATE_COMMUNICATOR_RETRIES exceeded");

    // reverse count of errors that can occur
    // - on each success IT recovers one point
    // - on each failure IT looses one point
    let mut measurement_errors_left = MAX_CONSECUTIVE_MEASUREMENT_ERRORS_ALLOWED;
    let mut post_failures_left = MAX_SEND_DATA_LOOP_FAILED_ON_POST_DATA_ERRORS_ALLOWED;

    // main loop, measuring
    'measure: while !(measurement_errors_left == 0) || !(post_failures_left == 0) {
        // -- Measure
        let measurement = match sensors.measure() {
            Ok((ret_sensors, measurement)) => {
                sensors = ret_sensors;
                if measurement_errors_left < MAX_CONSECUTIVE_MEASUREMENT_ERRORS_ALLOWED {
                    measurement_errors_left += 1;
                }
                measurement
            }
            Err((ret_sensors, ())) => {
                log::error!("Error measuring");
                sensors = ret_sensors;
                measurement_errors_left -= 1;
                continue 'measure;
            }
        };
        // --

        // -- Post the new data
        let serialized_data =
            serde_json::to_string(&measurement.data).expect("Data should be serializable");
        let data = PostSensorData {
            serialized_data,
            created_at: None,
        };

        for _ in 0..POST_DATA_RETRIES {
            // A post data retry doesn't count as failure as ESP Wifi
            // fails many times, so we don't panic so easily for it
            log::info!("Trying to send data to server");
            print_heap_and_stack();
            match communicator.post(&data) {
                Ok(ret_data) => {
                    log::info!("Correctly sent sensor_data: {ret_data:?}");
                    if post_failures_left < MAX_SEND_DATA_LOOP_FAILED_ON_POST_DATA_ERRORS_ALLOWED {
                        post_failures_left += 1;
                    }
                    continue 'measure;
                }
                Err(e) => match e {
                    server_communicator::Error::UnexpectedResponse(c) => match c {
                        401 | 404 => handle_unauthorized(persistence),
                        _ => {
                            log::error!("Response was an error on post data: {e:?}");
                        }
                    },
                    e => {
                        log::error!("Error got on post data: {e:?}")
                    }
                },
            }
        }
        post_failures_left -= 1;
        // --
    }
    panic!("Errors exceeded");
}

fn handle_unauthorized(mut persistence: Persistence) -> ! {
    log::warn!("Received UNAUTHORIZED code from server, emptying persistence and rebooting");
    if !persistence
        .remove(crate::persistence::Keys::AuthKeysSerialized)
        .expect("Should not fail to remove AuthKeysSerialized")
    {
        log::error!("Didn't remove any key for AuthKeysSerialized")
    }
    if !persistence
        .remove(crate::persistence::Keys::WifiConfigSerialized)
        .expect("Should not fail to remove WifiConfigSerialized")
    {
        log::error!("Didn't remove any key for WifiConfigSerialized")
    }
    panic!("Cleared persistence")
}

fn initial_config(device_id: DeviceId, mut persistence: Persistence) -> ! {
    // Run the initial config
    let keys = auth::keys::Keys::new(&get_random_buf());

    let config = match BleInitialConfigImplementation::run(device_id, &keys) {
        Ok(r) => r,
        Err(e) => {
            log::error!("BleLoop error: {e:?}");
            panic!("BleLoop")
        }
    };

    {
        // Wifi Config Save
        let ssid_string = config
            .get_wifi_ssid()
            .expect("The BLE must have configured a valid SSID");
        let mut ssid = heapless::String::new();
        ssid.push_str(&ssid_string)
            .expect("SSID should fit in 32 bytes");

        let pass_string = config
            .get_wifi_pass()
            .expect("The BLE must have configured a valid Wifi Password");
        let mut pass = heapless::String::new();
        pass.push_str(&pass_string)
            .expect("pass should fit in 32 bytes");

        // TODO: Get rid of hardcoded auth_method
        let wifi_config = WifiClientConfig {
            ssid,
            auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
            password: pass,
        };

        persistence
            .set(
                persistence::Keys::WifiConfigSerialized,
                &serde_json::to_string(&wifi_config).expect("wifi_config should be serializable"),
            )
            .expect("persistence.set should not fail on wifi_config");
    }

    {
        // auth_keys Save
        let auth_keys: Keys = keys;
        persistence
            .set(
                persistence::Keys::AuthKeysSerialized,
                &serde_json::to_string(&auth_keys).expect("auth_keys should be serializable"),
            )
            .expect("persistence.set should not fail on auth_keys");
    }

    // Once saved, restart
    panic!("Persistence should be correctly set when reboot"); // Somethat dirty to do it
}

fn get_device_id() -> DeviceId {
    let mut device_id = [0u8; 20];
    device_id[..6].copy_from_slice(&get_device_mac());
    DeviceId::from_string(&ZStr20::new(&device_id).as_hex_string()).expect("Should be valid")
}

fn get_device_mac() -> [u8; 6] {
    let mut mac: [u8; 6] = [0; 6];
    // Read the MAC address
    unsafe {
        esp_read_mac(mac.as_mut_ptr(), esp_idf_sys::esp_mac_type_t_ESP_MAC_BT);
    }

    log::debug!("get_device_mac: {:?}", mac);

    mac
}

pub fn print_heap_and_stack() {
    unsafe {
        // Heap (bytes)
        let free_heap: u32 = esp_get_free_heap_size();
        let min_free_heap: u32 = esp_get_minimum_free_heap_size();

        // Stack high-water mark of the current task (bytes on ESP-IDF)
        // Pass null to query the calling task per ESP-IDF docs
        let hwm_bytes = uxTaskGetStackHighWaterMark(core::ptr::null_mut());

        // Print using println! or the log crate if configured
        log::info!(
            "Free heap: {} B, Min free heap: {} B, Stack HWM (current task): {} B",
            free_heap,
            min_free_heap,
            hwm_bytes
        );
    }
}
