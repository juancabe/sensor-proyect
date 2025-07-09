use esp32_nimble::{utilities::BleUuid, uuid128, NimbleProperties};

const CFG_SERVICE_UUID: &str = "4b80ba9d-64fd-4ffa-86fb-544e73d26ed1";
const SENSOR_API_ID_CHAR_UUID: &str = "8c680060-22b7-45b8-b325-f7b1b102d80f";
const API_ACCOUNT_ID_CHAR_UUID: &str = "e11ca181-20c9-4675-b6f3-3f9fb91d1dc1";
const SENSOR_UUID_CHAR_UUID: &str = "333cad84-ceb5-4e18-bfcf-6147987c6733";

#[derive(Debug, Clone, Copy)]
pub struct BleCharacteristic<'a, R, W, N> {
    pub uuid: &'a BleUuid,
    pub read: Option<fn(R) -> [u8; 20]>,
    pub write: Option<fn(&[u8]) -> Option<W>>,
    pub notify: Option<fn(N) -> [u8; 20]>,
}

impl<'a, R, W, N> BleCharacteristic<'a, R, W, N> {
    pub fn get_nimble_properties(&self) -> NimbleProperties {
        let mut properties = NimbleProperties::empty();
        if self.read.is_some() {
            properties.insert(NimbleProperties::READ);
        }
        if self.write.is_some() {
            properties.insert(NimbleProperties::WRITE);
        }
        if self.notify.is_some() {
            properties.insert(NimbleProperties::NOTIFY);
        }
        properties
    }
}

pub struct BleProtocol<'a> {
    pub service_uuid: &'a BleUuid,
    pub characteristics: (
        BleCharacteristic<'a, (), ZStr20, ()>,
        BleCharacteristic<'a, (), ZStr20, ()>,
        BleCharacteristic<'a, (), (), &'a [u8; 20]>,
    ),
}

const SENSOR_CONFIG_SERVICE_UUID: BleUuid = uuid128!(CFG_SERVICE_UUID);
const SENSOR_CONFIG_SENSOR_API_ID_CHAR_UUID: BleUuid = uuid128!(SENSOR_API_ID_CHAR_UUID);
const SENSOR_CONFIG_API_ACCOUNT_ID_CHAR_UUID: BleUuid = uuid128!(API_ACCOUNT_ID_CHAR_UUID);
const SENSOR_CONFIG_SENSOR_UUID_CHAR_UUID: BleUuid = uuid128!(SENSOR_UUID_CHAR_UUID);

impl<'a> BleProtocol<'a> {
    pub fn new(sensor_id: fn(()) -> [u8; 20]) -> Self {
        BleProtocol {
            service_uuid: &SENSOR_CONFIG_SERVICE_UUID,
            characteristics: (
                BleCharacteristic {
                    uuid: &SENSOR_CONFIG_SENSOR_API_ID_CHAR_UUID,
                    read: None,
                    write: Some(ZStr20::from_unsized_slice),
                    notify: None,
                },
                BleCharacteristic {
                    uuid: &SENSOR_CONFIG_API_ACCOUNT_ID_CHAR_UUID,
                    read: None,
                    write: Some(ZStr20::from_unsized_slice),
                    notify: None,
                },
                BleCharacteristic {
                    uuid: &SENSOR_CONFIG_SENSOR_UUID_CHAR_UUID,
                    read: Some(sensor_id),
                    write: None,
                    notify: None,
                },
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZStr20 {
    pub bytes: [u8; 21],
}

impl ZStr20 {
    const SIZE: usize = 20;

    pub fn new(bytes: &[u8; Self::SIZE]) -> Self {
        let mut zstr = ZStr20 { bytes: [0; 21] };
        zstr.bytes[..Self::SIZE].copy_from_slice(bytes);
        zstr.bytes[Self::SIZE] = 0; // Null-terminate
        zstr
    }

    pub fn from_unsized_slice(slice: &[u8]) -> Option<Self> {
        // if slice.len() == Self::SIZE {
        //     let arr: [u8; Self::SIZE] = slice
        //         .try_into()
        //         .expect("Slice length was not what was expected");
        //     Some(ZStr20::new(&arr))
        // } else {
        //     None
        // }

        Some(ZStr20::new(slice.try_into().ok()?))
    }
}
