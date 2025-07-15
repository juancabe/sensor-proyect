use embedded_svc::wifi::Wifi;
use esp_idf_svc::wifi::{BlockingWifi, NonBlocking};
use esp_idf_sys::{esp, EspError};
use esp_idf_sys::{esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_MIN_MODEM, wifi_ps_type_t_WIFI_PS_NONE};

pub trait ModemSleep {
    fn modem_sleep(&mut self) -> Result<(), EspError>;
    fn modem_wakeup(&mut self) -> Result<(), EspError>;
}

impl<T> ModemSleep for BlockingWifi<T>
where
    T: Wifi<Error = EspError> + NonBlocking,
{
    fn modem_sleep(&mut self) -> Result<(), EspError> {
        // enable modem-sleep
        esp!(unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_MIN_MODEM) })?;
        Ok(())
    }
    fn modem_wakeup(&mut self) -> Result<(), EspError> {
        // turn power save off
        esp!(unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE) })?;
        Ok(())
    }
}
