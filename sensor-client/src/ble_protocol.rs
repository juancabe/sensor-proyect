use std::sync::{Arc, Mutex};

use common::{
    auth::keys::Keys,
    ble_protocol::BleInitalConfig,
    types::{
        validate::device_id::DeviceId,
        zstr20::{ZStr20, ZStr20Error},
    },
};
use esp32_nimble::{
    utilities::BleUuid, uuid128, BLEAdvertisementData, BLEDevice, BLEError, NimbleProperties,
};
use esp_idf_svc::hal::delay::FreeRtos;

const SENSOR_CONFIG_SERVICE_UUID: BleUuid = uuid128!("4b80ba9d-64fd-4ffa-86fb-544e73d26ed1");
const WRITE_WIFI_SSID_0: BleUuid = uuid128!("141ae9a4-f662-425f-b1b5-5bb35a9e043f");
const WRITE_WIFI_SSID_1: BleUuid = uuid128!("4b928144-f17a-478f-ab0e-c2c1b5ffad7a");
const WRITE_WIFI_PASS_0: BleUuid = uuid128!("7ccf2c21-1708-4189-b4c1-c09b3bf45f9d");
const WRITE_WIFI_PASS_1: BleUuid = uuid128!("52765c89-cfc0-45b6-89da-10d4d6eeb8ec");
const WRITE_WIFI_PASS_2: BleUuid = uuid128!("77b2f6d8-3ebc-46a6-85b3-0c01251d2430");
const WRITE_WIFI_PASS_3: BleUuid = uuid128!("32581ebc-3766-431d-bd2f-390ce6bee0c9");
const READ_SENSOR_DEVICE_ID: BleUuid = uuid128!("7af24399-6c3f-4bc3-b576-7a4f8fb59d41");
const READ_PUB_KEY_0: BleUuid = uuid128!("f4f1c584-e3c0-4723-9e46-1f72b015aa88");
const READ_PUB_KEY_1: BleUuid = uuid128!("7d725923-ca33-41c6-9a15-821be70eac7d");

pub struct BleInitialConfigImplementation;

#[derive(Debug)]
#[allow(dead_code)]
pub enum BleInitialConfigError {
    BleAdvertiserSetData(BLEError),
    BleAdvertiserStart(BLEError),
    IncorrectZStr20(ZStr20Error),
    MutexLock(String),
}

impl BleInitialConfigImplementation {
    pub fn run(device_id: DeviceId, keys: &Keys) -> Result<BleInitalConfig, BleInitialConfigError> {
        let ble_device = BLEDevice::take();
        let result_mutex = Arc::new(Mutex::new(BleInitalConfig::default()));

        let device_id = ZStr20::from_hex_string(device_id.as_str())
            .map_err(|e| BleInitialConfigError::IncorrectZStr20(e))?;
        let (pk0, pk1) = {
            let pk = keys.get_vk();
            let zstrs = ZStr20::arbitrary_bytes_to_multiple(&pk);
            assert!(zstrs.len() == 2);
            let pk0 = zstrs[0];
            let pk1 = zstrs[1];
            (pk0, pk1)
        };

        Self::configure_server(&result_mutex, &ble_device, device_id, pk0, pk1)?;

        let advertiser = ble_device.get_advertising();
        advertiser
            .lock()
            .set_data(
                BLEAdvertisementData::new()
                    .name(&device_id.as_hex_string())
                    .add_service_uuid(SENSOR_CONFIG_SERVICE_UUID),
            )
            .map_err(|e| BleInitialConfigError::BleAdvertiserSetData(e))?;

        advertiser
            .lock()
            .start()
            .map_err(|e| BleInitialConfigError::BleAdvertiserStart(e))?;

        log::info!("[run] BLE Advertiser started");

        loop {
            FreeRtos::delay_ms(200);
            let result = result_mutex
                .lock()
                .map_err(|e| BleInitialConfigError::MutexLock(e.to_string()))?;
            if result.is_written() {
                log::info!("[run] Results written successfully: {:?}", result);
                return Ok(result.clone());
            }
        }
    }

    fn configure_server(
        result_mutex: &Arc<Mutex<BleInitalConfig>>,
        ble_device: &BLEDevice,
        device_id: ZStr20,
        pub_key_0: ZStr20,
        pub_key_1: ZStr20,
    ) -> Result<(), BleInitialConfigError> {
        let server = ble_device.get_server();
        let svc = server.create_service(SENSOR_CONFIG_SERVICE_UUID);

        log::info!(
            "Setting READ_SENSOR_DEVICE_ID to: {}",
            device_id.as_hex_string()
        );
        // READ_SENSOR_DEVICE_ID
        svc.lock()
            .create_characteristic(READ_SENSOR_DEVICE_ID, NimbleProperties::READ)
            .lock()
            .on_read(|_, _| {
                log::info!("[READ_SENSOR_DEVICE_ID] called");
            })
            .set_value(device_id.inner_info_slice());

        log::info!("Setting READ_PUB_KEY_0 to: {}", pub_key_0.as_hex_string());
        // READ_PUB_KEY_0
        svc.lock()
            .create_characteristic(READ_PUB_KEY_0, NimbleProperties::READ)
            .lock()
            .on_read(|_, _| {
                log::info!("[READ_PUB_KEY_0] called");
            })
            .set_value(pub_key_0.inner_info_slice());

        log::info!("Setting READ_PUB_KEY_1 to: {}", pub_key_1.as_hex_string());
        // READ_PUB_KEY_1
        svc.lock()
            .create_characteristic(READ_PUB_KEY_1, NimbleProperties::READ)
            .lock()
            .on_read(|_, _| {
                log::info!("[READ_PUB_KEY_1] called");
            })
            .set_value(pub_key_1.inner_info_slice());

        // WRITE_WIFI_SSID_0
        let result_clone = result_mutex.clone();
        svc.lock()
            .create_characteristic(WRITE_WIFI_SSID_0, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_SSID_0] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(ssid) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_SSID_0] Error unlocking result on characteristic ")
                            .wifi_ssid0 = Some(ssid)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_SSID_0] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        // WRITE_WIFI_SSID_1
        let result_clone = result_mutex.clone();
        svc.lock()
            .create_characteristic(WRITE_WIFI_SSID_1, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_SSID_1] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(ssid) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_SSID_1] Error unlocking result on characteristic ")
                            .wifi_ssid1 = Some(ssid)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_SSID_1] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        //WRITE_WIFI_PASS_0
        let result_clone = result_mutex.clone();
        svc.lock()
            .create_characteristic(WRITE_WIFI_PASS_0, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_PASS_0] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(zstr) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_PASS_0] Error unlocking result on characteristic ")
                            .wifi_pass_0 = Some(zstr)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_PASS_0] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        //WRITE_WIFI_PASS_1
        let result_clone = result_mutex.clone();

        svc.lock()
            .create_characteristic(WRITE_WIFI_PASS_1, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_PASS_1] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(zstr) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_PASS_1] Error unlocking result on characteristic ")
                            .wifi_pass_1 = Some(zstr)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_PASS_1] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        //WRITE_WIFI_PASS_2
        let result_clone = result_mutex.clone();

        svc.lock()
            .create_characteristic(WRITE_WIFI_PASS_2, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_PASS_2] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(zstr) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_PASS_2] Error unlocking result on characteristic")
                            .wifi_pass_2 = Some(zstr)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_PASS_2] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        //WRITE_WIFI_PASS_3
        let result_clone = result_mutex.clone();
        svc.lock()
            .create_characteristic(WRITE_WIFI_PASS_3, NimbleProperties::WRITE)
            .lock()
            .on_write(move |args| {
                log::info!("[WRITE_WIFI_PASS_3] called");
                match ZStr20::from_unsized_slice(args.recv_data()) {
                    Ok(zstr) => {
                        result_clone
                            .lock()
                            .expect("[WRITE_WIFI_PASS_3] Error unlocking result on characteristic")
                            .wifi_pass_3 = Some(zstr)
                    }
                    Err(e) => {
                        log::error!(
                            "[WRITE_WIFI_PASS_3] Couldn't form ZStr20 from received data: {e:?}"
                        )
                    }
                }
            });

        Ok(())
    }
}
