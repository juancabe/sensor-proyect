use common::{
    auth::keys::Keys,
    endpoints_io::{
        sensor_data::PostSensorData,
        session::{ApiSession, PostSession, SensorLogin},
    },
    types::validate::device_id::{self, DeviceId},
};
use embedded_svc::http::client::Client;
use esp_idf_svc::{
    http::client::{Configuration, EspHttpConnection},
    io::Write,
};
use esp_idf_sys::esp_crt_bundle_attach;
use http::StatusCode;

use crate::helpers::get_random_buf;

const BASE_URL: &str = "http://192.168.1.139:3000/api/v0";

const SESSION_POST_RESPONSE_SIZE: usize = 2_000;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Deserialization(serde_json::Error),
    Serialization(serde_json::Error),
    HttpCreation(esp_idf_sys::EspError),
    DeviceId(device_id::Error),
    RequestCreation(esp_idf_svc::io::EspIOError),
    RequestWrite(esp_idf_svc::io::EspIOError),
    RequestSubmission(esp_idf_svc::io::EspIOError),
    UnexpectedResponse(u16),
    ErrorReadingResponse(esp_idf_svc::io::EspIOError),
}

#[allow(dead_code)]
pub struct ServerCommunicator {
    jwt: String,
    device_id: String,
    http_client: Client<EspHttpConnection>,
}

impl ServerCommunicator {
    pub fn generate(key: &mut Keys, device_id: DeviceId) -> Result<Self, Error> {
        let mut http_conf = Configuration::default();
        http_conf.crt_bundle_attach = Some(esp_crt_bundle_attach);

        let client = EspHttpConnection::new(&http_conf).map_err(|e| Error::HttpCreation(e))?;
        let mut client = Client::wrap(client);

        let url = format!("{BASE_URL}/session");
        let random_message: [u8; 64] = get_random_buf();
        let signature_of_message = hex::encode(key.sign(&random_message).to_bytes());
        let random_message = hex::encode(random_message);

        let body = PostSession::Sensor(SensorLogin {
            device_id: device_id.clone(),
            signature_of_message,
            random_message_encoded: random_message,
        });

        log::info!("Parsing request body for url: {url}");

        let request_body = match serde_json::to_string(&body) {
            Ok(body) => body,
            Err(e) => Err(Error::Serialization(e))?,
        };

        log::info!("Initializing post request for url: {url}");

        let resp = match client.post(
            &url,
            &[
                ("accept", "application/json"),
                ("Content-Type", "application/json"),
            ],
        ) {
            Ok(mut req) => {
                if let Err(e) = req.write_all(request_body.as_bytes()) {
                    Err(Error::RequestWrite(e))?
                } else {
                    req.submit()
                }
            }
            Err(e) => Err(Error::RequestCreation(e))?,
        };

        let sess = match resp {
            Ok(mut r) => {
                if r.status() != StatusCode::OK {
                    Err(Error::UnexpectedResponse(r.status()))?
                }

                let mut buffer = [0u8; SESSION_POST_RESPONSE_SIZE];
                let read = r
                    .read(buffer.as_mut_slice())
                    .map_err(|e| Error::ErrorReadingResponse(e))?;
                log::info!("Read {read} bytes from server");
                let session: ApiSession = serde_json::from_slice(buffer.as_mut_slice())
                    .map_err(|e| Error::Deserialization(e))?;
                log::info!("Got session: {session:?}");
                session
            }
            Err(e) => Err(Error::RequestSubmission(e))?,
        };

        log::info!("ServerCommunicator correctly generated");

        Ok(Self {
            jwt: sess.access_token,
            device_id: device_id.to_string(),
            http_client: client,
        })
    }

    pub fn post(_data: PostSensorData) -> Result<(), ()> {
        todo!()
    }
}
