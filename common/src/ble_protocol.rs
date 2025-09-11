use crate::types::zstr20::ZStr20;

#[derive(Default, Clone, Copy, Debug)]
pub struct BleInitalConfig {
    pub wifi_ssid0: Option<ZStr20>,
    pub wifi_ssid1: Option<ZStr20>,
    pub wifi_pass_0: Option<ZStr20>,
    pub wifi_pass_1: Option<ZStr20>,
    pub wifi_pass_2: Option<ZStr20>,
    pub wifi_pass_3: Option<ZStr20>,
}

impl BleInitalConfig {
    pub fn get_wifi_ssid(&self) -> Result<String, ()> {
        match (self.wifi_ssid0, self.wifi_ssid1) {
            (Some(first), Some(second)) => {
                let null_index = first
                    .bytes
                    .iter()
                    .position(|element| *element == 0u8)
                    .unwrap_or(first.bytes.len());
                let first_to_null = &first.bytes.as_slice()[..null_index];
                let first_string = match String::from_utf8(Vec::from(first_to_null)) {
                    Ok(string) => string,
                    Err(e) => {
                        log::error!("Error first string: {e:?}");
                        return Err(());
                    }
                };
                log::info!("the first string is: {first_string}");

                if null_index != first.bytes.len() {
                    log::info!("Second part of WIFI_SSID unused due to NULL found in first");
                    return Ok(first_string);
                }

                let null_index = second
                    .bytes
                    .iter()
                    .position(|element| *element == 0u8)
                    .unwrap_or(second.bytes.len());

                let second_to_null = &second.bytes.as_slice()[..null_index];
                let second_string = match String::from_utf8(Vec::from(second_to_null)) {
                    Ok(string) => string,
                    Err(e) => {
                        log::error!("Error second string: {e:?}");
                        return Err(());
                    }
                };

                Ok(first_string + &second_string)
            }
            _ => Err(()),
        }
    }

    pub fn get_wifi_pass(&self) -> Result<String, ()> {
        match (
            self.wifi_pass_0,
            self.wifi_pass_1,
            self.wifi_pass_2,
            self.wifi_pass_3,
        ) {
            (Some(p1), Some(p2), Some(p3), Some(p4)) => {
                let mut vec =
                    ZStr20::multiple_to_arbitrary_bytes(&[p1, p2, p3, p4], ZStr20::LENGTH * 4);
                let null_index = vec.iter().position(|e| *e == 0u8).unwrap_or(vec.len());
                vec.truncate(null_index);
                String::from_utf8(vec).map_err(|e| {
                    log::error!("Error forming string from wifi password parts: {e:?}");
                })
            }
            _ => Err(()),
        }
    }

    pub fn is_written(&self) -> bool {
        log::info!("BleInitalConfig: {:?}", self);

        self.wifi_ssid0.is_some()
            && self.wifi_ssid1.is_some()
            && self.wifi_pass_0.is_some()
            && self.wifi_pass_1.is_some()
            && self.wifi_pass_2.is_some()
            && self.wifi_pass_3.is_some()
    }
}

#[cfg(test)]
mod test {

    use crate::{ble_protocol::BleInitalConfig, types::zstr20::ZStr20};

    #[test]
    fn test_get_wifi_ssid_valid() {
        let wifi_ssid0 = ZStr20::new(&[
            119, 105, 102, 105, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let wifi_ssid0 = Some(wifi_ssid0);
        let wifi_ssid1 = ZStr20::new(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let wifi_ssid1 = Some(wifi_ssid1);
        let config = BleInitalConfig {
            wifi_ssid0,
            wifi_ssid1,
            wifi_pass_0: None,
            wifi_pass_1: None,
            wifi_pass_2: None,
            wifi_pass_3: None,
        };
        let wifi = config.get_wifi_ssid().expect("Should work");
        assert_eq!(wifi, "wifi");
    }

    #[test]
    fn more_test() {
        todo!();
    }
}
