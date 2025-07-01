use core::str::FromStr;
use embedded_svc::http::client::Client;
use esp_idf_svc::{
    hal::{
        delay::FreeRtos,
        i2c::{I2cConfig, I2cDriver},
        peripherals,
        prelude::*,
    },
    http::{
        client::{Configuration as HttpConfig, EspHttpConnection},
        Method,
    },
    io::Write,
    sys::{esp_get_free_heap_size, uxTaskGetStackHighWaterMark},
    wifi::{
        AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi, PmfConfiguration,
        ScanMethod, ScanSortMethod,
    },
};
use sensor_lib::api::{
    self, endpoints::post_aht10_data::PostAht10DataBody, model::aht10_data::Aht10Data, ApiEndpoint,
};
use serde_json::Value;

fn log_memory_usage(task_name: &str) {
    let free_heap = unsafe { esp_get_free_heap_size() };
    let stack_high_water_mark = unsafe { uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
    log::info!(
        "{}: Free heap: {} bytes, Stack high water mark: {} bytes",
        task_name,
        free_heap,
        stack_high_water_mark
    );
}

fn read_aht10_data(aht10: &mut adafruit_aht10::AdafruitAHT10<I2cDriver>) {
    match aht10.read_data() {
        Ok((humidity, temperature)) => {
            log::info!(
                "Temperature: {:.2} °C, Humidity: {:.2} %",
                temperature,
                humidity
            );
        }
        Err(e) => log::error!("Failed to read data from AHT10: {:?}", e),
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log_memory_usage("Start of main");

    log::info!("Hello, world!");

    let peripherals = peripherals::Peripherals::take().expect("Failed to take peripherals");
    log_memory_usage("Before WiFi Initialization");

    let modem = peripherals.modem;
    let sysloop = esp_idf_svc::eventloop::EspEventLoop::take().expect("Failed to take event loop");
    log_memory_usage("After Event Loop Initialization");

    let nvs =
        esp_idf_svc::nvs::EspDefaultNvsPartition::take().expect("Failed to take NVS partition");
    log_memory_usage("After NVS Partition Initialization");

    let wifi =
        EspWifi::new(modem, sysloop.clone(), Some(nvs.clone())).expect("Failed to create WiFi");
    log_memory_usage("After WiFi Initialization");

    let mut wifi = BlockingWifi::wrap(wifi, sysloop).expect("Failed to wrap WiFi");
    log_memory_usage("After Wrapping WiFi");

    // let sda = peripherals.pins.gpio3;
    // let scl = peripherals.pins.gpio2;

    // let i2c = peripherals.i2c0;
    // let config = I2cConfig::new().baudrate(100.kHz().into());
    // let aht10 = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    // let mut aht10: adafruit_aht10::AdafruitAHT10<_> = adafruit_aht10::AdafruitAHT10::new(aht10);

    // match aht10.begin() {
    //     Ok(()) => log::info!("AHT10 initialized successfully!"),
    //     Err(e) => log::error!("Failed to initialize AHT10: {:?}", e),
    // }

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::from_str("Pixel").unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: heapless::String::from_str("1111").unwrap(),
        channel: None,
        scan_method: ScanMethod::CompleteScan(ScanSortMethod::Signal),
        pmf_cfg: PmfConfiguration::NotCapable,
    }))
    .unwrap();

    log_memory_usage("Before WiFi Start");

    match wifi.start() {
        Ok(()) => log::info!("WiFi started successfully!"),
        Err(e) => log::error!("Failed to start WiFi: {:?}", e),
    }

    match wifi.connect() {
        Ok(()) => log::info!("Connected to WiFi!"),
        Err(e) => log::error!("Failed to connect to WiFi: {:?}", e),
    }

    // Wait until the network interface is up
    match wifi.wait_netif_up() {
        Ok(()) => log::info!("Network interface is up!"),
        Err(e) => log::error!("Failed to wait for network interface: {:?}", e),
    }

    // Print Out Wifi Connection Configuration
    while !wifi.is_connected().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    log::info!("WiFi connection established!");

    // HTTP Configuration
    // Create HTTPS Connection Handle
    let httpconnection = EspHttpConnection::new(&HttpConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        timeout: Some(core::time::Duration::from_secs(10)),
        ..Default::default()
    })
    .unwrap_or_else(|e| {
        log::error!("Failed to create HTTP connection: {:?}", e);
        panic!("HTTP connection creation failed");
    });

    let base_url = "http://sensor-server.juancb.ftp.sh:3000";

    // Create HTTPS Client
    let mut httpclient = Client::wrap(httpconnection);

    loop {
        let request_body: PostAht10DataBody = PostAht10DataBody {
            user_uuid: "test_user_uuid".to_string(),
            user_place_id: 1,
            data: Aht10Data {
                sensor_id: "sensor_123".to_string(),
                humidity: 12.23,
                temperature: 23.45,
            },
            added_at: Some(chrono::Utc::now().timestamp()),
        };

        let request_body = match serde_json::to_string(&request_body) {
            Ok(body) => body,
            Err(e) => {
                log::error!("Failed to serialize request body: {:?}", e);
                FreeRtos::delay_ms(1000);
                continue;
            }
        };

        let url = format!(
            "{}{}",
            base_url,
            api::endpoints::post_aht10_data::PostAht10::PATH
        );

        log::info!("-> Sending HTTP POST request to {}", url);

        let resp = match httpclient.post(&url, &[("accept", "application/json")]) {
            Ok(mut req) => {
                if let Err(e) = req.write_all(request_body.as_bytes()) {
                    log::error!("Failed to write request body: {:?}", e);
                    FreeRtos::delay_ms(1000);
                    continue;
                } else {
                    log::info!("Request body written successfully");
                    req.submit()
                }
            }
            Err(e) => {
                log::error!("Failed to create HTTP POST request: {:?}", e);
                FreeRtos::delay_ms(1000);
                continue;
            }
        };

        let mut resp = match resp {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to send HTTP POST request: {:?}", e);
                FreeRtos::delay_ms(1000);
                continue;
            }
        };

        log::info!("<- HTTP Response Code: {:?}", resp.status());

        let mut buf = [0u8; 1024];

        match resp.read(&mut buf) {
            Ok(size) => {
                log::info!(
                    "<- HTTP Response Body [{} bytes], {}",
                    size,
                    core::str::from_utf8(&buf[..size]).unwrap()
                );

                let json = serde_json::from_slice::<Value>(&buf[..size]);
                match json {
                    Ok(json) => {
                        if let Some(_json) = json.get("json") {
                            log::info!("JSON found in response");
                        } else {
                            log::info!("JSON not found in response");
                            log::info!("Full JSON response: {:?}", json);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse JSON response: {:?}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read HTTP response body: {:?}", e);
            }
        }

        FreeRtos::delay_ms(1000);
    }
}
