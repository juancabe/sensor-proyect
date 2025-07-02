use esp_idf_svc::wifi::AuthMethod;

pub type SSID = &'static str;
pub type Password = &'static str;

pub const CLIENT_WIFIS: [(SSID, Password, AuthMethod); 2] = [
    ("WifiSsid1", "1111", AuthMethod::WPA2Personal),
    ("WifiSsid2", "1111", AuthMethod::WPA2Personal),
];
