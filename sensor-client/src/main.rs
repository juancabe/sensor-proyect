use core::{panic, str::FromStr};
use embedded_svc::http::client::Client;
use esp32_nimble::{utilities::mutex::Mutex, BLEAdvertisementData, BLEDevice};
use esp_idf_svc::{
    eventloop::System,
    hal::{delay::FreeRtos, i2c::I2cDriver, peripherals, prelude::*},
    http::client::{Configuration as HttpConfiguration, EspHttpConnection},
    io::Write,
    sys::{esp_get_free_heap_size, uxTaskGetStackHighWaterMark},
    wifi::{
        BlockingWifi, ClientConfiguration, Configuration, EspWifi, PmfConfiguration, ScanMethod,
        ScanSortMethod,
    },
};
use esp_idf_sys::esp_read_mac;
use sensor_lib::api::{
    endpoints::post_sensor_data::{PostSensorData, PostSensorDataBody, PostSensorResponseCode},
    model::{
        // aht10_data::Aht10Data,
        any_sensor_data::AnySensorData,
        scd4x_data::Scd4xData,
        SensorData,
    },
    ApiEndpoint,
};
use std::sync::Arc;

use crate::{
    ble_protocol::ZStr20,
    modem_sleep::ModemSleep,
    persistence::{Keys, Persistence},
    sensors::{Scd41InitData, Scd41WorkingMode, Sensors},
};

mod ble_protocol;
mod modem_sleep;
mod persistence;
mod private;
mod sensors;

const BASE_URL: &'static str = "http://sensor-server.juancb.ftp.sh:3000";
const READS_DELAY_MS: u32 = 1000 * 60; // 1 minute

const HTTP_POST_TRIES: u32 = 10;

fn _log_memory_usage(task_name: &str) {
    let free_heap = unsafe { esp_get_free_heap_size() };
    let stack_high_water_mark = unsafe { uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
    log::info!(
        "{}: Free heap: {} bytes, Stack high water mark: {} bytes",
        task_name,
        free_heap,
        stack_high_water_mark
    );
}

struct I2CPeripherals {
    gpio3_sda: esp_idf_svc::hal::gpio::Gpio4,
    gpio2_scl: esp_idf_svc::hal::gpio::Gpio3,
    i2c0: esp_idf_svc::hal::i2c::I2C0,
}

#[allow(dead_code)]
#[derive(Debug)]
enum SendAht10DataError {
    SerializationError(serde_json::Error),
    RequestCreationError(esp_idf_svc::io::EspIOError),
    RequestWriteError(esp_idf_svc::io::EspIOError),
    RequestSubmissionError(esp_idf_svc::io::EspIOError),
    UnexpectedResponseError(u16),
}

fn send_sensor_data(
    (user_api, sensor_api): (&str, &str),
    client: &mut Client<EspHttpConnection>,
    data: AnySensorData,
) -> Result<PostSensorResponseCode, SendAht10DataError> {
    let url: String = format!("{}{}", BASE_URL, PostSensorData::PATH);

    let body = PostSensorDataBody {
        user_api_id: user_api.to_string(),
        sensor_api_id: sensor_api.to_string(),
        data,
        added_at: None,
    };

    let request_body = match serde_json::to_string(&body) {
        Ok(body) => body,
        Err(e) => Err(SendAht10DataError::SerializationError(e))?,
    };

    let resp = match client.post(&url, &[("accept", "application/json")]) {
        Ok(mut req) => {
            if let Err(e) = req.write_all(request_body.as_bytes()) {
                Err(SendAht10DataError::RequestWriteError(e))?
            } else {
                req.submit()
            }
        }
        Err(e) => Err(SendAht10DataError::RequestCreationError(e))?,
    };

    let resp_status = match resp {
        Ok(response) => response,
        Err(e) => Err(SendAht10DataError::RequestSubmissionError(e))?,
    }
    .status();

    let r = PostSensorResponseCode::from_u16(resp_status)
        .map_err(|e| SendAht10DataError::UnexpectedResponseError(e))?;

    Ok(r)
}

struct WifiDevices {
    modem: esp_idf_svc::hal::modem::Modem,
    sysloop: esp_idf_svc::eventloop::EspEventLoop<System>,
    nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,
}

#[allow(dead_code)]
#[derive(Debug)]
enum LoadWifiError {
    WifiCreationError(esp_idf_sys::EspError),
    WifiWrapError(esp_idf_sys::EspError),
    ConfigurationError(esp_idf_sys::EspError),
    StartError(esp_idf_sys::EspError),
    ConnectError(esp_idf_sys::EspError),
    WaitNetifUpError(esp_idf_sys::EspError),
    NotConnected,
}

fn load_wifi_device(
    wifi_devices: WifiDevices,
) -> Result<BlockingWifi<EspWifi<'static>>, LoadWifiError> {
    let wifi = match EspWifi::new(
        wifi_devices.modem,
        wifi_devices.sysloop.clone(),
        Some(wifi_devices.nvs.clone()),
    ) {
        Ok(wifi) => wifi,
        Err(e) => {
            log::error!("Failed to create WiFi: {:?}", e);
            Err(LoadWifiError::WifiCreationError(e))?
        }
    };

    match BlockingWifi::wrap(wifi, wifi_devices.sysloop) {
        Ok(wifi) => Ok(wifi),
        Err(e) => {
            log::error!("Failed to wrap WiFi: {:?}", e);
            Err(LoadWifiError::WifiWrapError(e))
        }
    }
}

fn connect_wifi<'a>(
    wifi_device: &mut BlockingWifi<EspWifi<'a>>,
    client_config: ClientConfiguration,
    timeout_ms: u32,
) -> Result<(), LoadWifiError> {
    wifi_device
        .set_configuration(&Configuration::Client(client_config))
        .map_err(|e| LoadWifiError::ConfigurationError(e))?;

    wifi_device
        .start()
        .map_err(|e| LoadWifiError::StartError(e))?;

    wifi_device
        .connect()
        .map_err(|e| LoadWifiError::ConnectError(e))?;

    FreeRtos::delay_ms(1000);

    // Wait until the network interface is up
    wifi_device
        .wait_netif_up()
        .map_err(|e| LoadWifiError::WaitNetifUpError(e))?;

    const TIMES: u32 = 10;
    let delay = timeout_ms / TIMES;
    let mut attempts = 0;

    while !wifi_device.is_connected().unwrap() {
        attempts += 1;
        FreeRtos::delay_ms(delay);
        log::warn!(
            "Wifi not connected yet, retying [{}/{}] attempts",
            attempts,
            TIMES
        );
        if attempts >= TIMES {
            log::error!("Failed to connect to WiFi after {} attempts", TIMES);
            Err(LoadWifiError::NotConnected)?
        }
    }

    Ok(())
}

fn wait_wifi_stopped(
    wifi_device: &mut BlockingWifi<EspWifi>,
    milliseconds: u32,
) -> Result<(), esp_idf_sys::EspError> {
    wifi_device.modem_sleep()?;
    log::info!("WiFi sleeping successfully");
    FreeRtos::delay_ms(milliseconds);
    log::info!("WiFi waking up...");
    wifi_device.modem_wakeup()?;
    wifi_device.wait_netif_up()?;
    log::info!("Reconnected to WiFi");
    Ok(())
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

fn return_sensor_id(_: ()) -> [u8; 20] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&get_device_mac());
    let sid = hasher.finalize()[..20] // We can subslice into 20 bytes because a SHA256 hash is made of 256 bits == 32 bytes
        .try_into()
        .expect("Array of length 20 should have 20 items");

    log::debug!("return_sensor_id: {:?}", sid);
    sid
}

#[derive(Default, Clone, Copy)]
struct BleLoopReturn {
    pub user_api_id: Option<ZStr20>,
    pub sensor_api_id: Option<ZStr20>,
}

impl BleLoopReturn {
    pub fn is_written(&self) -> bool {
        self.sensor_api_id.is_some() && self.user_api_id.is_some()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum BleLoopError {
    BleAdvertiserLockError(String),
}

fn configure_esp_ble_loop(ble_device: &BLEDevice) -> Result<BleLoopReturn, BleLoopError> {
    use ble_protocol;
    let p = ble_protocol::BleProtocol::new(return_sensor_id);

    let ble_advertiser = ble_device.get_advertising();
    let server = ble_device.get_server();
    let svc = server.create_service(*p.service_uuid);

    let result_mutex = Arc::new(Mutex::new(BleLoopReturn::default()));

    let (sensor_api_id_char, api_account_id_char, sensor_uuid_char, _wifi_ssid, _wifi_pass) =
        p.characteristics;
    // TODO: Wifi pass and wifi ssid characteristics

    let result_clone = result_mutex.clone();
    svc.lock()
        .create_characteristic(
            *sensor_api_id_char.uuid,
            sensor_api_id_char.get_nimble_properties(),
        )
        .lock()
        .on_write(move |args| {
            let write_fn = sensor_api_id_char
                .write
                .expect("Characteristic write function is not set");

            log::info!(
                "Characteristic [sensor_api_id_char] written: {}",
                String::from_utf8_lossy(args.current_data())
            );

            let data = args.recv_data();
            if let Some(zstr) = write_fn(data) {
                log::info!("Received ZStr20: {:?}", zstr);
                result_clone.lock().sensor_api_id = Some(zstr);
            } else {
                log::error!("Failed to parse ZStr20 from data: {:?}", data);
            }
        });

    let result_clone = result_mutex.clone();
    svc.lock()
        .create_characteristic(
            *api_account_id_char.uuid,
            sensor_api_id_char.get_nimble_properties(),
        )
        .lock()
        .on_write(move |args| {
            let write_fn = api_account_id_char
                .write
                .expect("Characteristic write function is not set");

            log::info!(
                "Characteristic {} written: {}",
                sensor_api_id_char.uuid,
                String::from_utf8_lossy(args.current_data())
            );

            let data = args.recv_data();
            if let Some(zstr) = write_fn(data) {
                log::info!("Received ZStr20: {:?}", zstr);
                result_clone.lock().user_api_id = Some(zstr);
            } else {
                log::error!("Failed to parse ZStr20 from data: {:?}", data);
            }
        });

    let val = sensor_uuid_char.read.expect("Read function not set")(());
    let val_print = sensor_uuid_char.read.expect("Read function not set")(());
    svc.lock()
        .create_characteristic(
            *sensor_uuid_char.uuid,
            sensor_uuid_char.get_nimble_properties(),
        )
        .lock()
        .on_read(move |_args, _args2| {
            log::info!("UUID characteristic read, value: {:?}", val_print)
        })
        .set_value(&val);

    ble_advertiser
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("esp32-sensor")
                .add_service_uuid(*p.service_uuid),
        )
        .map_err(|e| BleLoopError::BleAdvertiserLockError(e.to_string()))?;

    ble_advertiser
        .lock()
        .start()
        .map_err(|e| BleLoopError::BleAdvertiserLockError(e.to_string()))?;

    log::info!("BLE advertising started");
    log::info!("BLE service created with UUID: {}", *p.service_uuid);

    loop {
        FreeRtos::delay_ms(200);

        // Check if the service is still running
        if !ble_advertiser.lock().is_advertising() {
            log::error!("BLE advertising stopped, restarting...");
            ble_advertiser
                .lock()
                .start()
                .map_err(|e| BleLoopError::BleAdvertiserLockError(e.to_string()))?;
        }

        if result_mutex.lock().is_written() {
            log::info!("Results written successfully");
            let _ = ble_advertiser
                .lock()
                .stop()
                .inspect_err(|e| log::warn!("Error stopping ble_adverstiser: {:?}", e));
            return Ok(*(result_mutex.lock()));
        }
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = peripherals::Peripherals::take().expect("Failed to take peripherals");

    let config = esp_idf_svc::hal::i2c::I2cConfig::new().baudrate(100.kHz().into());
    let i2cp = I2CPeripherals {
        gpio3_sda: peripherals.pins.gpio4,
        gpio2_scl: peripherals.pins.gpio3,
        i2c0: peripherals.i2c0,
    };

    let i2c = I2cDriver::new(i2cp.i2c0, i2cp.gpio3_sda, i2cp.gpio2_scl, &config)
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

    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()
        .expect("Couldn't take NVS Default Partition");
    let mut persistence: Persistence =
        Persistence::new(nvs.clone()).expect("Error creating Persistence");

    let sensor_key: persistence::Keys = persistence::Keys::SensorApiId(None);
    let user_key: persistence::Keys = persistence::Keys::UserApiId(None);

    let mut sensor_buf = vec![0u8; sensor_key.min_buffer_size()];
    let mut user_buf = vec![0u8; user_key.min_buffer_size()];

    if !(persistence
        .get(&sensor_key, &mut sensor_buf)
        .expect("Should be able to use NVS [SensorApiId]")
        && persistence
            .get(&user_key, &mut user_buf)
            .expect("Should be able to use NVS [UserApiId]"))
    {
        // If any neccessary data for sever communication is not available, we set it with BLE
        let ble_device = BLEDevice::take();
        let res: BleLoopReturn;
        let mut count = 0;
        loop {
            if count > 100 {
                panic!("Too many retries for configure_esp_ble_loop")
            }

            match configure_esp_ble_loop(&ble_device) {
                Ok(ret) => {
                    res = ret;
                    break;
                }
                Err(e) => {
                    log::warn!(
                        "An error occurred on the configure_esp_ble_loop, retrying: {:?}",
                        e
                    );
                    count += 1;
                }
            }
        }

        let ble_sensor_api_id = res.sensor_api_id.expect("Should be Some");
        let ble_user_api_id = res.user_api_id.expect("Should be Some");

        log::debug!(
            "\n[RAW BLE]\nSensor API ID: {:?}\nUser API ID: {:?}",
            ble_sensor_api_id.bytes,
            ble_user_api_id.bytes
        );

        let sensor_api_hex = res.sensor_api_id.expect("Should be Some").as_hex_string();
        let user_api_hex = res.user_api_id.expect("Should be Some").as_hex_string();
        let sensor_key = persistence::Keys::SensorApiId(Some(&sensor_api_hex));
        let user_key = persistence::Keys::UserApiId(Some(&user_api_hex));

        log::info!(
            "\nSensor API ID: {}\nUser API ID: {}",
            sensor_api_hex,
            user_api_hex
        );
        persistence
            .set(&sensor_key)
            .expect("Unable to set SensorApiId value");
        persistence
            .set(&user_key)
            .expect("Unable to set UserApiId value");

        let a = persistence
            .get(&sensor_key, &mut sensor_buf)
            .expect("Should be able to use NVS [SensorApiId]");
        let b = persistence
            .get(&user_key, &mut user_buf)
            .expect("Should be able to use NVS [UserApiId]");

        if !(a && b) {
            panic!("NVS Keys-Values were not there after they were set")
        }
    }

    let user_api =
        String::from_utf8(user_buf).expect("Should be able to convert user_buf to string");
    let sens_api =
        String::from_utf8(sensor_buf).expect("Should be able to convert sensor_buf to string");

    log::info!("user_api.len(): {}, user_api: {}", user_api.len(), user_api);
    log::info!("sens_api.len(): {}, sens_api: {}", sens_api.len(), sens_api);

    let sysloop = esp_idf_svc::eventloop::EspEventLoop::take().expect("Failed to take event loop");

    let modem = peripherals.modem;

    let wifi_devices = WifiDevices {
        modem,
        sysloop: sysloop.clone(),
        nvs: nvs.clone(),
    };

    let mut wifi_device = load_wifi_device(wifi_devices).expect("Failed to load WiFi device");

    let http_client = EspHttpConnection::new(&HttpConfiguration::default())
        .expect("Failed to create HTTP client");
    let mut http_client = Client::wrap(http_client);

    let mut wifi_attempts_failed: usize = 0;

    let sensor_uuid: String = ZStr20::new(&return_sensor_id(())).as_hex_string();

    'select_wifi: loop {
        if wifi_attempts_failed >= private::CLIENT_WIFIS.len() * 3 {
            log::error!(
                "Failed to connect to WiFi after {} attempts",
                wifi_attempts_failed
            );
            // TODO: Cannot connect to any WiFi
            panic!("Cannot connect to any WiFi");
        }

        for wifi in private::CLIENT_WIFIS {
            log::info!("Attempting to connect to WiFi: {}", wifi.0);

            let client_config = ClientConfiguration {
                ssid: heapless::String::from_str(wifi.0).unwrap(),
                bssid: None,
                auth_method: wifi.2,
                password: heapless::String::from_str(wifi.1).unwrap(),
                channel: None,
                scan_method: ScanMethod::CompleteScan(ScanSortMethod::Signal),
                pmf_cfg: PmfConfiguration::NotCapable,
            };

            match connect_wifi(
                &mut wifi_device,
                client_config,
                500, // Timeout in milliseconds
            ) {
                Ok(()) => {
                    wifi_attempts_failed = 0;
                    log::info!("WiFi {} connected successfully", wifi.0);
                    loop {
                        let data = match sensors.measure() {
                            Ok((sensors_back, measurement)) => {
                                log::info!("Sensors measurement: {:?}", measurement);
                                sensors = sensors_back;
                                if !measurement.scd41_measured {
                                    None
                                } else {
                                    let d = Scd4xData {
                                        sensor_id: sensor_uuid.clone(),
                                        co2: measurement.data.co2.expect("CO2 should exist"),
                                        humidity: measurement
                                            .data
                                            .humidity
                                            .expect("Humidity should exist"),
                                        temperature: measurement
                                            .data
                                            .temperature
                                            .expect("Temperature should exist"),
                                    };
                                    Some(d)
                                }
                            }
                            Err((sensors_back, _e)) => {
                                sensors = sensors_back;
                                log::error!("No sensors measured");
                                None
                            }
                        };

                        // Send data to server
                        match data {
                            Some(data) => {
                                let mut data_sent = false;
                                for i in 0..HTTP_POST_TRIES {
                                    match send_sensor_data(
                                        (&user_api, &sens_api),
                                        &mut http_client,
                                        data.clone().to_any_sensor_data(),
                                    ) {
                                        Ok(response_code) => {
                                            log::info!(
                                                "Data sent successfully, response code: {:?}",
                                                response_code
                                            );

                                            if let PostSensorResponseCode::Unauthorized =
                                                response_code
                                            {
                                                match persistence.set(&Keys::SensorApiId(None)) {
                                                    Err(e) => log::error!(
                                                        "Error 'resetting' SensorApiId: {:?}",
                                                        e
                                                    ),
                                                    Ok(()) => log::info!("SensorApiId set to None"),
                                                }
                                                match persistence.set(&Keys::UserApiId(None)) {
                                                    Err(e) => log::error!(
                                                        "Error 'resetting' UserApiId: {:?}",
                                                        e
                                                    ),
                                                    Ok(()) => log::info!("UserApiId set to None"),
                                                }
                                                panic!("UNAUTHORIZED API KEYS");
                                            }

                                            data_sent = true;
                                            break;
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Failed to send Sensor data [{}/{}]: {:?}",
                                                i,
                                                HTTP_POST_TRIES,
                                                e
                                            );
                                        }
                                    }
                                }

                                if !data_sent {
                                    log::error!(
                                        "Failed to send AHT10 data after {} attempts",
                                        HTTP_POST_TRIES
                                    );
                                    continue 'select_wifi;
                                }
                            }
                            None => {
                                log::warn!("No Sensor data available");
                                // TODO: Send `sensor down` to server
                                continue 'select_wifi;
                            }
                        }

                        // Wait for a while before the next reading
                        match wait_wifi_stopped(&mut wifi_device, READS_DELAY_MS) {
                            Ok(()) => (),
                            Err(e) => {
                                log::error!("WiFi related error on wait_wifi_stopped: {:?}\nGoing back to select_wifi", e);
                                continue 'select_wifi;
                            }
                        }
                    }
                }
                Err(e) => {
                    wifi_attempts_failed += 1;
                    log::warn!("Failed to connect WiFi {}: {:?}", wifi.0, e);
                }
            }
        }
    }
}
