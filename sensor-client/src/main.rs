use std::str::FromStr;

use crate::{
    ble_protocol::BleInitalConfig,
    helpers::{get_random_buf, zstr20::ZStr20},
    server_communicator::ServerCommunicator,
    wifi_connector::WifiConnector,
};
use common::{auth::keys::Keys, types::validate::device_id::DeviceId};
use esp_idf_svc::{
    eventloop::EspEventLoop,
    hal::prelude::*,
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration},
};
use esp_idf_sys::esp_read_mac;

pub mod ble_protocol;
pub mod helpers;
pub mod persistence;
pub mod private;
pub mod server_communicator;
pub mod wifi_connector;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take().unwrap();
    let sysloop = EspEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().expect("Couldn't take NVS Default Partition");

    let device_id = get_device_id();
    log::info!("device_id: {device_id:?}");

    let configs = private::CLIENT_WIFIS.map(|(ssid, password, method)| {
        Configuration::Client(ClientConfiguration {
            ssid: heapless::String::from_str(ssid).unwrap(),
            bssid: None,
            auth_method: method,
            password: heapless::String::from_str(password).unwrap(),
            channel: None,
            scan_method: esp_idf_svc::wifi::ScanMethod::CompleteScan(
                esp_idf_svc::wifi::ScanSortMethod::Signal,
            ),
            pmf_cfg: esp_idf_svc::wifi::PmfConfiguration::NotCapable,
        })
    });

    let mut keys = Keys::new(&get_random_buf());

    let _state = match BleInitalConfig::run(device_id, &keys) {
        Ok(r) => r,
        Err(e) => {
            log::error!("BleLoop error: {e:?}");
            return;
        }
    };

    let _wifi = WifiConnector::from_configs(2, configs.as_slice(), p.modem, sysloop, nvs.clone())
        .expect("Should connect to Wifi");

    let comm = ServerCommunicator::generate(&mut keys, DeviceId::random());

    match comm {
        Ok(_c) => todo!(),
        Err(e) => log::error!("Error server comunicator: {e:?}"),
    }
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
