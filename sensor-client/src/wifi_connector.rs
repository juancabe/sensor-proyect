use esp_idf_svc::{
    eventloop::System,
    nvs::{EspNvsPartition, NvsDefault},
    wifi::{BlockingWifi, Configuration, EspWifi},
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    ConnectedCheck(esp_idf_sys::EspError),
    WifiCreation(esp_idf_sys::EspError),
    WifiWrap(esp_idf_sys::EspError),
    Configuration(esp_idf_sys::EspError),
    Start(esp_idf_sys::EspError),
    Connect(esp_idf_sys::EspError),
    WaitNetifUp(esp_idf_sys::EspError),
    NotConnected,
}

pub struct WifiConnector {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl WifiConnector {
    pub fn from_configs(
        retries_per_config: usize,
        configs: &[Configuration],
        modem: esp_idf_svc::hal::modem::Modem,
        sysloop: esp_idf_svc::eventloop::EspEventLoop<System>,
        nvs: EspNvsPartition<NvsDefault>,
    ) -> Result<WifiConnector, Error> {
        let mut device = Self::load_wifi_device(modem, sysloop, nvs)?;
        for config in configs {
            log::info!("Connecting to wifi: {:?}", config);
            match Self::connect_wifi(&mut device, config, retries_per_config) {
                Ok(()) => return Ok(Self { wifi: device }),
                Err(e) => {
                    log::error!("Could not connect to wifi, {e:?}");
                }
            }
        }
        Err(Error::NotConnected)
    }

    pub fn as_wifi(&self) -> &BlockingWifi<EspWifi<'static>> {
        &self.wifi
    }

    pub fn into_wifi(self) -> BlockingWifi<EspWifi<'static>> {
        self.wifi
    }

    pub fn from_connected_wifi(wifi: BlockingWifi<EspWifi<'static>>) -> Result<Self, Error> {
        wifi.wait_netif_up().map_err(|e| Error::WaitNetifUp(e))?;

        if wifi.is_connected().map_err(|e| Error::ConnectedCheck(e))? {
            Ok(Self { wifi })
        } else {
            Err(Error::NotConnected)
        }
    }

    fn load_wifi_device(
        modem: esp_idf_svc::hal::modem::Modem,
        sysloop: esp_idf_svc::eventloop::EspEventLoop<System>,
        nvs: EspNvsPartition<NvsDefault>,
    ) -> Result<BlockingWifi<EspWifi<'static>>, Error> {
        let wifi = match EspWifi::new(modem, sysloop.clone(), Some(nvs)) {
            Ok(wifi) => wifi,
            Err(e) => {
                log::error!("Failed to create WiFi: {:?}", e);
                Err(Error::WifiCreation(e))?
            }
        };

        let wifi = match BlockingWifi::wrap(wifi, sysloop) {
            Ok(wifi) => Ok(wifi),
            Err(e) => {
                log::error!("Failed to wrap WiFi: {:?}", e);
                Err(Error::WifiWrap(e))
            }
        };

        wifi
    }

    fn connect_wifi<'a>(
        wifi_device: &mut BlockingWifi<EspWifi<'a>>,
        config: &Configuration,
        // timeout_ms: u32,
        retries: usize,
    ) -> Result<(), Error> {
        wifi_device
            .set_configuration(config)
            .map_err(|e| Error::Configuration(e))?;

        wifi_device.start().map_err(|e| Error::Start(e))?;

        let mut connect = || -> Result<bool, Error> {
            wifi_device.connect().map_err(|e| Error::Connect(e))?;

            // FreeRtos::delay_ms(1000);

            // Wait until the network interface is up
            wifi_device
                .wait_netif_up()
                .map_err(|e| Error::WaitNetifUp(e))?;

            Ok(wifi_device
                .is_connected()
                .map_err(|e| Error::ConnectedCheck(e))?)
        };

        for attempts in 1..=retries {
            match connect() {
                Ok(true) => {
                    return Ok(());
                }
                Ok(false) => {
                    log::warn!(
                        "[connect_wifi] connect() waited but wifi was not connected, retrying"
                    );
                }
                Err(e) => {
                    log::warn!(
                        "Wifi not connected yet, retrying [{}/{}] attempts | e: {e:?}",
                        attempts,
                        retries
                    );
                }
            }
        }

        Err(Error::NotConnected)
    }
}
