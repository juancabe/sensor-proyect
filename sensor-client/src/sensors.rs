use std::fmt::Debug;

use adafruit_aht10::AdafruitAHT10;
use esp_idf_svc::hal::{delay::FreeRtos, i2c::I2cDriver};
use esp_idf_sys::esp_timer_get_time;
use scd4x::Scd4x;
use serde::{Deserialize, Serialize};

// TODO: Get rid of this constant
const SENSOR_READ_TRIES: u32 = 10;

#[derive(Debug, Clone, Copy)]
pub struct Aht10WorkingData {}
impl Aht10WorkingData {
    pub fn new() -> Self {
        Aht10WorkingData {}
    }
}

#[derive(Debug)]
pub enum Aht10Status {
    NoWorkingDataSet,
    Created(Aht10WorkingData),
    InitFailed(InitAht10Error<'static>),
}

#[derive(Debug)]
pub struct Scd41InitData {
    mode: Scd41WorkingMode,
}

impl Scd41InitData {
    pub fn new(mode: Scd41WorkingMode) -> Self {
        Scd41InitData { mode }
    }
}

#[derive(Debug)]
pub enum Scd41WorkingMode {
    LowPower,
    Normal,
}

impl Scd41WorkingMode {
    pub fn measurement_timeout_millis(&self) -> i64 {
        match self {
            Scd41WorkingMode::LowPower => 30_000,
            Scd41WorkingMode::Normal => 5_000,
        }
    }
}

#[derive(Debug)]
pub struct Scd41WorkingData {
    mode: Scd41WorkingMode,
    // serial: Serial,
    last_measurement: i64,
}

impl Scd41WorkingData {
    fn get_time() -> i64 {
        unsafe { esp_timer_get_time() }
    }

    pub fn new(
        // serial: Serial,
        mode: Scd41WorkingMode,
    ) -> Self {
        Scd41WorkingData {
            // serial,
            last_measurement: Self::get_time(),
            mode,
        }
    }

    pub fn measured(&mut self) {
        self.last_measurement = Self::get_time();
    }

    pub fn millis_until_next_measurement(&self) -> i64 {
        let now = Self::get_time();
        let millis_since_last = (now - self.last_measurement) / 1_000;
        self.mode.measurement_timeout_millis() - millis_since_last
    }

    pub fn wait_until_measurement(&self) {
        let millis = self.millis_until_next_measurement();

        if millis > 0 {
            FreeRtos::delay_ms(millis as u32)
        }
    }
}

#[derive(Debug)]
pub enum Scd41Status {
    NoWorkingDataSet,
    Created(Scd41WorkingData),
    InitFailed(InitScd41Error<'static>),
}

/// Contains runtime information for sensors lifecycle
/// Use Sensors::measure for measuring
pub struct Sensors<'a> {
    pub i2c: I2cDriver<'a>,
    pub aht10: Aht10Status,
    pub scd41: Scd41Status,
}

#[derive(Debug)]
pub enum SensorsError {}

pub type Serial = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorsData {
    pub co2: Option<u16>,
    pub humidity: Option<f32>,
    pub temperature: Option<f32>,
}

#[derive(Debug)]
pub struct SensorsMeasurement {
    pub aht10_measured: bool,
    pub scd41_measured: bool,
    pub data: SensorsData,
}

impl<'a> Sensors<'a> {
    pub fn new(
        i2c: I2cDriver<'_>,
        aht10data: Option<()>,
        scd41data: Option<Scd41InitData>,
        allow_partial_sensor: bool,
    ) -> Result<Sensors<'_>, (I2cDriver<'_>, Scd41Status, Aht10Status)> {
        const TO_BUILD_SENSORS: u32 = 2;

        let mut available = TO_BUILD_SENSORS;
        let scd41;
        let aht10;

        let i2c = match scd41data {
            None => {
                scd41 = Scd41Status::NoWorkingDataSet;
                i2c
            }
            Some(d) => match init_scd41_sensor(i2c) {
                Ok((i2c, _serial)) => {
                    scd41 = Scd41Status::Created(Scd41WorkingData::new(
                        // serial,
                        d.mode,
                    ));
                    i2c
                }
                Err(err_enum) => {
                    let (e, i2c) = err_enum.destructure();
                    scd41 = Scd41Status::InitFailed(e);
                    available -= 1;
                    i2c.expect("I2cDriver should be available after init_scd41_sensor")
                }
            },
        };

        let i2c = match aht10data {
            None => {
                aht10 = Aht10Status::NoWorkingDataSet;
                i2c
            }
            Some(_d) => match init_aht10_sensor(i2c) {
                Ok(i2c) => {
                    aht10 = Aht10Status::Created(Aht10WorkingData::new());
                    i2c
                }
                Err(err_enum) => {
                    let (e, i2c) = err_enum.destructure();
                    aht10 = Aht10Status::InitFailed(e);
                    available -= 1;
                    i2c.expect("I2cDriver should be available after init_scd41_sensor")
                }
            },
        };

        if available == 0 {
            Err((i2c, scd41, aht10))
        } else if available < TO_BUILD_SENSORS && !allow_partial_sensor {
            Err((i2c, scd41, aht10))
        } else {
            Ok(Sensors { i2c, aht10, scd41 })
        }
    }

    pub fn init_error(&self) -> Result<(), u16> {
        let mut errors = 0;

        if let Aht10Status::InitFailed(_) = self.aht10 {
            errors += 1;
        }

        if let Scd41Status::InitFailed(_) = self.scd41 {
            errors += 1;
        }

        if errors != 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn measure(mut self) -> Result<(Self, SensorsMeasurement), (Self, ())> {
        let mut aht10_measured = false;
        let mut scd41_measured = false;
        let mut data = SensorsData {
            co2: None,
            humidity: None,
            temperature: None,
        };

        self.i2c = match self.scd41 {
            Scd41Status::Created(ref mut working_data) => {
                let mut scd41 = Scd4x::new(self.i2c, FreeRtos);
                for _ in 0..SENSOR_READ_TRIES {
                    let is_ready = match scd41.data_ready_status() {
                        Ok(b) => b,
                        Err(_i2c) => {
                            continue;
                        }
                    };
                    if !is_ready {
                        working_data.wait_until_measurement();
                    }

                    working_data.wait_until_measurement();
                    match scd41.measurement() {
                        Ok(sensor_data) => {
                            let co2 = sensor_data.co2;
                            let temperature = sensor_data.temperature;
                            let humidity = sensor_data.humidity;

                            if temperature < -41.0 {
                                log::warn!(
                                    "[SCD41] Temperature reading is below -41.0 °C, skipping this reading"
                                );
                                continue;
                            }

                            data.co2 = Some(co2);
                            data.humidity = Some(humidity);
                            data.temperature = Some(temperature);
                            scd41_measured = true;
                            working_data.measured();

                            log::info!("[SCD41] data: {:?}", data);

                            break;
                        }
                        Err(e) => {
                            log::warn!("[SCD41] Failed to read data: {:?}", e);
                        }
                    };
                }
                scd41.destroy()
            }
            _ => self.i2c,
        };

        self.i2c = match self.aht10 {
            Aht10Status::Created(_) => {
                let mut aht10 = AdafruitAHT10::new(self.i2c);
                for _ in 0..SENSOR_READ_TRIES {
                    match aht10.read_data() {
                        Ok((humidity, temperature)) => {
                            if temperature < -41.0 {
                                log::warn!(
                                    "[AHT10] Temperature reading is below -41.0 °C, skipping this reading"
                                );
                                continue;
                            }

                            // AHT10 temp and humidity sensor is better so we keep it
                            data.humidity = Some(humidity);
                            data.temperature = Some(temperature);
                            aht10_measured = true;

                            log::info!("[AHT10] data: {:?}", data);

                            break;
                        }
                        Err(e) => {
                            log::warn!("[AHT10] Failed to read data: {:?}", e);
                        }
                    };
                }
                aht10.destroy()
            }
            _ => self.i2c,
        };

        if !aht10_measured && !scd41_measured {
            return Err((self, ()));
        } else {
            Ok((
                self,
                SensorsMeasurement {
                    aht10_measured,
                    scd41_measured,
                    data,
                },
            ))
        }
    }
}

// HELPERS

#[allow(mismatched_lifetime_syntaxes)]
fn init_scd41_sensor(i2c: I2cDriver<'_>) -> Result<(I2cDriver<'_>, Serial), InitScd41Error> {
    let delay = esp_idf_svc::hal::delay::FreeRtos;
    let mut scd41 = Scd4x::new(i2c, delay);

    log::info!("SCD41 sensor waking up...");
    scd41.wake_up();

    if let Err(e) = scd41.stop_periodic_measurement() {
        return Err(InitScd41Error::StopPeriodicMesurement((
            format!("SCD_ERROR: {:?}", e),
            Some(scd41.destroy()),
        )));
    }

    if let Err(e) = scd41.reinit() {
        return Err(InitScd41Error::Reinit((
            format!("SCD_ERROR: {:?}", e),
            Some(scd41.destroy()),
        )));
    }

    let serial = match scd41.serial_number() {
        Ok(s) => s,
        Err(e) => {
            return Err(InitScd41Error::SerialNumber((
                format!("SCD_ERROR: {:?}", e),
                Some(scd41.destroy()),
            )))
        }
    };
    log::trace!("SCD41 serial: {:#04x}", serial);

    match scd41.self_test_is_ok() {
        Ok(true) => log::info!("SCD41 self-test passed"),
        Ok(false) => log::error!("SCD41 self-test failed"),
        Err(e) => {
            return Err(InitScd41Error::SelfTestOk((
                format!("SCD_ERROR: {:?}", e),
                Some(scd41.destroy()),
            )))
        }
    }

    if let Err(e) = scd41.start_low_power_periodic_measurements() {
        return Err(InitScd41Error::StartLowPowerPeriodicMeasurement((
            format!("SCD_ERROR: {:?}", e),
            Some(scd41.destroy()),
        )));
    }

    println!("Waiting for first measurement... (5 sec)");
    Ok((scd41.destroy(), serial))
}

pub enum InitScd41Error<'a> {
    StopPeriodicMesurement((String, Option<I2cDriver<'a>>)),
    Reinit((String, Option<I2cDriver<'a>>)),
    SerialNumber((String, Option<I2cDriver<'a>>)),
    SelfTestOk((String, Option<I2cDriver<'a>>)),
    StartPeriodicMeasurement((String, Option<I2cDriver<'a>>)),
    StartLowPowerPeriodicMeasurement((String, Option<I2cDriver<'a>>)),
}

impl<'a> InitScd41Error<'a> {
    pub fn destructure(self) -> (InitScd41Error<'static>, Option<I2cDriver<'a>>) {
        match self {
            InitScd41Error::StopPeriodicMesurement((err, i2c)) => {
                (InitScd41Error::StopPeriodicMesurement((err, None)), i2c)
            }
            InitScd41Error::Reinit((err, i2c)) => (InitScd41Error::Reinit((err, None)), i2c),
            InitScd41Error::SerialNumber((err, i2c)) => {
                (InitScd41Error::SerialNumber((err, None)), i2c)
            }
            InitScd41Error::SelfTestOk((err, i2c)) => {
                (InitScd41Error::SelfTestOk((err, None)), i2c)
            }
            InitScd41Error::StartPeriodicMeasurement((err, i2c)) => {
                (InitScd41Error::StartPeriodicMeasurement((err, None)), i2c)
            }
            InitScd41Error::StartLowPowerPeriodicMeasurement((err, i2c)) => (
                InitScd41Error::StartLowPowerPeriodicMeasurement((err, None)),
                i2c,
            ),
        }
    }
}

impl<'a> Debug for InitScd41Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StopPeriodicMesurement((arg0, _driver)) => {
                f.debug_tuple("StopPeriodicMesurement").field(arg0).finish()
            }
            Self::Reinit((arg0, _driver)) => f.debug_tuple("Reinit").field(arg0).finish(),
            Self::SerialNumber((arg0, _driver)) => {
                f.debug_tuple("SerialNumber").field(arg0).finish()
            }
            Self::SelfTestOk((arg0, _driver)) => f.debug_tuple("SelfTestOk").field(arg0).finish(),
            Self::StartPeriodicMeasurement((arg0, _driver)) => f
                .debug_tuple("StartPeriodicMeasurement")
                .field(arg0)
                .finish(),
            Self::StartLowPowerPeriodicMeasurement((arg0, _driver)) => f
                .debug_tuple("StartLowPowerPeriodicMeasurement")
                .field(arg0)
                .finish(),
        }
    }
}

#[allow(mismatched_lifetime_syntaxes)]
fn init_aht10_sensor(i2c: I2cDriver<'_>) -> Result<I2cDriver<'_>, InitAht10Error> {
    let mut aht10 = adafruit_aht10::AdafruitAHT10::new(i2c);

    if let Err(e) = aht10.begin() {
        return Err(InitAht10Error::InitializationError((
            e,
            Some(aht10.destroy()),
        )));
    }

    Ok(aht10.destroy())
}

pub enum InitAht10Error<'a> {
    I2cError((esp_idf_sys::EspError, Option<I2cDriver<'a>>)),
    InitializationError((adafruit_aht10::Aht10Error, Option<I2cDriver<'a>>)),
}

impl<'a> InitAht10Error<'a> {
    pub fn destructure(self) -> (InitAht10Error<'static>, Option<I2cDriver<'a>>) {
        match self {
            InitAht10Error::I2cError((e, d)) => (InitAht10Error::I2cError((e, None)), d),
            InitAht10Error::InitializationError((e, d)) => {
                (InitAht10Error::InitializationError((e, None)), d)
            }
        }
    }
}

impl<'a> Debug for InitAht10Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I2cError((arg0, _driver)) => f.debug_tuple("I2cError").field(arg0).finish(),
            Self::InitializationError((arg0, _driver)) => {
                f.debug_tuple("InitializationError").field(arg0).finish()
            }
        }
    }
}
