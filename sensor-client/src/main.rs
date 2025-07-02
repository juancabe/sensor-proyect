use core::{panic, str::FromStr};
use embedded_svc::http::client::Client;
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
use sensor_lib::api::{
    endpoints::post_aht10_data::{PostAht10, PostAht10DataBody, PostAht10ResponseCode},
    model::aht10_data::Aht10Data,
    ApiEndpoint,
};

mod private;

const BASE_URL: &'static str = "http://sensor-server.juancb.ftp.sh:3000";
const USER_UUID: &'static str = "test-user-uuid";
const USER_PLACE_ID: i32 = 1;
const SENSOR_UUID: &'static str = "test-sensor-uuid";
const READS_DELAY_MS: u32 = 1000 * 60; // 1 minute

const HTTP_POST_TRIES: u32 = 10;
const SENSOR_READ_TRIES: u32 = 10;

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

struct Aht10Peripherals {
    gpio3_sda: esp_idf_svc::hal::gpio::Gpio3,
    gpio2_scl: esp_idf_svc::hal::gpio::Gpio2,
    i2c0: esp_idf_svc::hal::i2c::I2C0,
}

#[allow(dead_code)]
#[derive(Debug)]
enum GetAht10Error {
    I2cError(esp_idf_sys::EspError),
    InitializationError(adafruit_aht10::Aht10Error),
}

fn get_aht10_sensor(
    peripherals: Aht10Peripherals,
) -> Result<adafruit_aht10::AdafruitAHT10<I2cDriver<'static>>, GetAht10Error> {
    let config = esp_idf_svc::hal::i2c::I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.gpio3_sda,
        peripherals.gpio2_scl,
        &config,
    )
    .map_err(|e| GetAht10Error::I2cError(e))?;

    let mut aht10 = adafruit_aht10::AdafruitAHT10::new(i2c);

    aht10
        .begin()
        .map_err(|e| GetAht10Error::InitializationError(e))?;

    Ok(aht10)
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

fn send_aht10_data(
    client: &mut Client<EspHttpConnection>,
    data: Aht10Data,
) -> Result<PostAht10ResponseCode, SendAht10DataError> {
    let url = format!("{}{}", BASE_URL, PostAht10::PATH);

    let body = PostAht10DataBody {
        user_uuid: USER_UUID.to_string(),
        user_place_id: USER_PLACE_ID,
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

    let r = PostAht10ResponseCode::from_u16(resp_status)
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
    wifi_device.disconnect()?;
    wifi_device.stop()?;
    log::info!("WiFi stopped successfully");
    FreeRtos::delay_ms(milliseconds);
    log::info!("Reconnecting to WiFi...");
    wifi_device.start()?;
    wifi_device.connect()?;
    wifi_device.wait_netif_up()?;
    log::info!("Reconnected to WiFi");
    Ok(())
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = peripherals::Peripherals::take().expect("Failed to take peripherals");

    let mut aht10 = get_aht10_sensor(Aht10Peripherals {
        gpio3_sda: peripherals.pins.gpio3,
        gpio2_scl: peripherals.pins.gpio2,
        i2c0: peripherals.i2c0,
    })
    .expect("Failed to get AHT10 sensor");

    let sysloop = esp_idf_svc::eventloop::EspEventLoop::take().expect("Failed to take event loop");

    let nvs =
        esp_idf_svc::nvs::EspDefaultNvsPartition::take().expect("Failed to take NVS partition");

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
                        let mut data: Option<Aht10Data> = None;

                        // Read data from AHT10 sensor
                        for _ in 0..SENSOR_READ_TRIES {
                            match aht10.read_data() {
                                Ok((humidity, temperature)) => {
                                    if temperature < -41.0 {
                                        log::warn!("Temperature reading is below -41.0 °C, skipping this reading");
                                        continue;
                                    }

                                    data = Some(Aht10Data {
                                        sensor_id: SENSOR_UUID.to_string(),
                                        humidity,
                                        temperature,
                                    });

                                    log::info!("AHT10 data: {:?}", data);

                                    break;
                                }
                                Err(e) => {
                                    log::warn!("Failed to read data from AHT10: {:?}", e);
                                }
                            };
                        }

                        // Send data to server
                        match data {
                            Some(data) => {
                                let mut data_sent = false;
                                for i in 0..HTTP_POST_TRIES {
                                    match send_aht10_data(&mut http_client, data.clone()) {
                                        Ok(response_code) => {
                                            log::info!(
                                                "Data sent successfully, response code: {:?}",
                                                response_code
                                            );
                                            data_sent = true;
                                            break;
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Failed to send AHT10 data [{}/{}]: {:?}",
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
                                log::warn!("No AHT10 data available");
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
