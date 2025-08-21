use axum::{Router, routing::MethodRouter};
use dotenv::dotenv;

use crate::{
    api::{Endpoint, endpoints::generate_endpoints},
    auth::keys::KEYS,
    db::establish_connection,
};

pub type ServerMethodRouter = MethodRouter;

pub struct SensorServer {
    endpoints: Vec<Box<dyn Endpoint>>,
}

impl SensorServer {
    pub const API_BASE: &str = "/api/v0";
    pub fn new() -> Self {
        // Load LazyStatics
        let _ = *KEYS;
        log::info!("Loaded keys for JWT");

        dotenv().expect(".env should be available and readable");

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        assert!(!database_url.contains("test"));
        establish_connection(false).expect("Connection should be available");
        log::info!("Loaded DB_POOL");

        let endpoints = generate_endpoints();

        Self { endpoints }
    }

    pub fn for_test() -> Self {
        // Load LazyStatics
        let _ = *KEYS;
        log::info!("Loaded keys for JWT");

        dotenv().expect(".env should be available and readable");

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        assert!(database_url.contains("test"));
        establish_connection(false).expect("Connection should be available");
        log::info!("Loaded DB_POOL");

        let endpoints = generate_endpoints();

        Self { endpoints }
    }

    pub fn routes(&self) -> impl Iterator<Item = (String, ServerMethodRouter)> {
        self.endpoints
            .iter()
            .map(|endpoint| endpoint.routes())
            .flatten()
            .map(|route| {
                (
                    String::from(Self::API_BASE) + route.path.as_str(),
                    route.method_router.clone(),
                )
            })
    }

    pub fn into_router(self) -> Router {
        let mut router = Router::new();

        for (path, route) in self.routes() {
            router = router.route(&path, route);
        }

        router.layer(axum::middleware::from_fn(crate::middleware::log_request))
    }
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use hyper::StatusCode;
    use serde_valid::json::json;

    use crate::{
        api::{
            endpoints::{
                self,
                place::{ApiUserPlace, GetPlace, GetPlaceEnum, PostPlace},
                sensor::{ApiUserSensor, GetSensor, GetSensorEnum, PostSensor},
                sensor_data::{
                    ApiSensorData, GetSensorData, PostSensorData, PostSensorDataResponse,
                },
                session::{ApiSession, GetSession},
                user::{ApiUser, GetUser, NotUniqueUser, PostUser},
            },
            types::{
                device_id::DeviceId,
                validate::{
                    api_color::ApiColor, api_description::ApiDescription, api_email::ApiEmail,
                    api_entity_name::ApiEntityName, api_raw_password::ApiRawPassword,
                    api_username::ApiUsername,
                },
            },
        },
        db::tests::random_string,
        model::COLOR_HEX_STRS,
        sensor_server::SensorServer,
    };

    // #[test]
    // #[ignore = "DB should not include test in name, must commit changes and then be reverted"]
    // fn test_sensor_server() {
    //     let _sensor_server = SensorServer::new();
    // }

    #[tokio::test]
    #[ignore = "reason"]
    // #[ignore = "DB should not include test in name, must commit changes and then be reverted"]
    async fn test_integration() {
        // This test should be run in a db that can be 'migration redo'
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .init();

        log::info!("Hello Test!");

        let mut server = TestServer::new(SensorServer::for_test().into_router())
            .expect("Should be created successfully");
        server.expect_success();

        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::health::Health::API_PATH
        );
        let res = server.get(path.as_str()).await;
        server.clear_query_params();
        res.assert_text("OK");

        // Create the user
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::user::User::API_PATH
        );

        let username = ApiUsername::random();
        let raw_password = ApiRawPassword::random();
        let email = ApiEmail::random();

        let body = PostUser {
            username: username.clone(),
            email: email.clone(),
            raw_password: raw_password.clone(),
        };

        server.post(path.as_str()).json(&body).await;

        // , same username: should fail
        let body = PostUser {
            username: username.clone(),
            email: ApiEmail::random(),
            raw_password: ApiRawPassword::random(),
        };
        let res = server
            .post(path.as_str())
            .json(&body)
            .expect_failure()
            .await;
        assert_eq!(StatusCode::CONFLICT, res.status_code());
        let resp: Option<NotUniqueUser> = res.json();
        assert_eq!(
            NotUniqueUser::Username(username.clone().into()),
            resp.unwrap()
        );

        // , same email: should fail
        let body = PostUser {
            username: ApiUsername::random(),
            email: email.clone(),
            raw_password: ApiRawPassword::random(),
        };
        let res = server
            .post(path.as_str())
            .json(&body)
            .expect_failure()
            .await;
        assert_eq!(StatusCode::CONFLICT, res.status_code());
        let resp: Option<NotUniqueUser> = res.json();
        assert_eq!(NotUniqueUser::Email(email.clone().into()), resp.unwrap());

        // , invalid username: should fail
        let body = PostUser {
            username: "a".to_string().into(),
            email: ApiEmail::random(),
            raw_password: ApiRawPassword::random(),
        };
        let res = server
            .post(path.as_str())
            .json(&body)
            .expect_failure()
            .await;
        assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, res.status_code());
        let resp: String = res.text();
        assert!(resp.contains("Invalid username"));

        // Get the session
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::session::Session::API_PATH
        );

        let query = GetSession {
            username: username.clone(),
            raw_password: raw_password.clone(),
        };

        let res = server
            .get(path.as_str())
            .add_query_params(json!(query))
            .await;
        let session: ApiSession = res.json();

        server.clear_query_params();
        server.add_header(
            "Authorization",
            (String::from("Bearer ") + session.access_token.as_str()).as_str(),
        );

        // Get User Back
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::user::User::API_PATH
        );

        let _query = GetUser {};
        let res = server.get(path.as_str()).await;
        server.clear_query_params();
        let api_user: ApiUser = res.json();

        assert_eq!(api_user.username, username);
        assert_eq!(api_user.email, email);
        // Add a place
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::place::Place::API_PATH
        );

        let name = ApiEntityName::random();
        let description = ApiDescription::random();
        let color = ApiColor::random();

        let body = PostPlace {
            name: name.clone(),
            description: Some(description.clone()),
            color: color.clone(),
        };
        let res = server.post(path.as_str()).json(&body).await;
        let api_place: ApiUserPlace = res.json();

        assert_eq!(api_place.name, name.clone());
        assert_eq!(
            api_place.description.expect("Should be set"),
            description.clone()
        );
        assert_eq!(api_place.color, color);

        // Add a place with same name should error
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::place::Place::API_PATH
        );

        let body = PostPlace {
            name: name.clone(),
            description: Some(description.clone().into()),
            color: COLOR_HEX_STRS[0].to_string().into(),
        };
        let res = server
            .post(path.as_str())
            .json(&body)
            .expect_failure()
            .await;

        assert_eq!(StatusCode::CONFLICT, res.status_code());

        // Add a sensor
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::sensor::Sensor::API_PATH
        );

        let sensor_name = ApiEntityName::random();
        let sensor_description = ApiDescription::random();
        let sensor_device_id = DeviceId::random();

        let body = PostSensor {
            name: sensor_name.clone().into(),
            description: Some(sensor_description.clone()),
            color: COLOR_HEX_STRS[0].to_string().into(),
            place_name: name.clone().into(),
            device_id: sensor_device_id.clone(),
        };

        let res = server.post(path.as_str()).json(&body).await;
        let api_sensor: ApiUserSensor = res.json();

        assert_eq!(api_sensor.name, sensor_name.clone().into());
        assert_eq!(api_sensor.device_id, sensor_device_id);
        assert_eq!(
            api_sensor.description.expect("Should be set"),
            sensor_description.clone().into()
        );
        assert_eq!(api_sensor.color, COLOR_HEX_STRS[0].to_string().into());

        // Add a sensor again, should fail
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::sensor::Sensor::API_PATH
        );

        let body = PostSensor {
            name: random_string(ApiEntityName::MIN_LEN..ApiEntityName::MAX_LEN).into(),
            description: Some(sensor_description.clone().into()),
            color: color.clone(),
            place_name: name.clone(),
            device_id: sensor_device_id.clone(),
        };

        let res = server
            .post(path.as_str())
            .json(&body)
            .expect_failure()
            .await;
        assert_eq!(StatusCode::CONFLICT, res.status_code());

        // Send sensor data
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::sensor_data::SensorData::API_PATH
        );

        let serialized_data = "
        {
            co2: 312
            temperature: 12
            humidity: 32
        }
        ";

        let body = PostSensorData {
            device_id: sensor_device_id.clone(),
            serialized_data: serialized_data.to_string(),
            created_at: None,
        };

        let res = server.post(&path).json(&body).await;
        let resp1: PostSensorDataResponse = res.json();

        // Send new data with the new JWT
        server.clear_headers();
        server.add_header(
            "Authorization",
            "Bearer ".to_string() + resp1.new_session.access_token.as_str(),
        );
        let res = server.post(&path).json(&body).await;
        let resp2: PostSensorDataResponse = res.json();

        // Get those added datas
        let query = GetSensorData {
            device_id: sensor_device_id.clone(),
            lowest_added_at: Some(resp1.api_data.added_at - 1),
            upper_added_at: Some(resp2.api_data.added_at + 1),
        };

        let res = server.get(&path).add_query_params(query).await;
        server.clear_query_params();

        let data: Vec<ApiSensorData> = res.json();
        assert_eq!(data.len(), 2);

        // Get those added datas without specifying range
        let query = GetSensorData {
            device_id: sensor_device_id.clone(),
            lowest_added_at: None,
            upper_added_at: None,
        };

        let res = server.get(&path).add_query_params(query).await;
        server.clear_query_params();

        let data: Vec<ApiSensorData> = res.json();
        assert_eq!(data.len(), 2);

        // Login with invalid password
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::session::Session::API_PATH
        );

        let query = GetSession {
            username: username.clone(),
            raw_password: ApiRawPassword::random(),
        };

        server.clear_headers();
        let res = server
            .get(&path)
            .add_query_params(query)
            .expect_failure()
            .await;

        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

        // Login with invalid username
        let path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::session::Session::API_PATH
        );

        let query = GetSession {
            username: ApiUsername::random(),
            raw_password: raw_password.clone(),
        };

        server.clear_headers();
        let res = server
            .get(&path)
            .add_query_params(query)
            .expect_failure()
            .await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

        server.clear_headers();
        server.add_header(
            "Authorization",
            "Bearer ".to_string() + resp2.new_session.access_token.as_str(),
        );

        // GET list of places
        let place_list_path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::place::Place::API_PATH
        );

        let query: GetPlace = GetPlace {
            param: GetPlaceEnum::FromPlaceName(name.clone()),
        };

        let res = server.get(&place_list_path).add_query_params(query).await;
        server.clear_query_params();
        let place_list: Vec<ApiUserPlace> = res.json();
        assert_eq!(place_list.len(), 1);
        let fetched_place = &place_list[0];
        assert_eq!(fetched_place.name, name);
        assert_eq!(fetched_place.description.as_ref().unwrap(), &description);
        assert_eq!(fetched_place.color, color.clone());

        // GET list of sensors
        let sensor_list_path = format!(
            "{}{}",
            SensorServer::API_BASE,
            endpoints::sensor::Sensor::API_PATH
        );
        let query = GetSensorEnum::FromPlaceName(name.clone());
        let query = GetSensor { param: query };

        let res = server.get(&sensor_list_path).add_query_params(query).await;
        server.clear_query_params();
        let sensor_list: Vec<ApiUserSensor> = res.json();
        assert_eq!(sensor_list.len(), 1);
        let fetched_sensor = &sensor_list[0];
        assert_eq!(
            &fetched_sensor.description.as_ref().unwrap().as_str(),
            &sensor_description.as_str()
        );
        assert_eq!(fetched_sensor.device_id, sensor_device_id);

        // Unauthorized access to protected endpoints
        server.clear_headers();
        let res = server.get(&place_list_path).expect_failure().await;
        assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);

        server.add_header("Authorization", "Bearer invalidjwt");
        let res = server.get(&sensor_list_path).expect_failure().await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

        let invalid_place = PostPlace {
            name: ApiEntityName::random(),
            description: None,
            color: color.clone(),
        };
        let res = server
            .post(&place_list_path)
            .json(&invalid_place)
            .expect_failure()
            .await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

        let invalid_sensor = PostSensor {
            name: ApiEntityName::random(),
            description: None,
            color: color.clone(),
            place_name: name.clone(),
            device_id: DeviceId::random(),
        };
        let res = server
            .post(&sensor_list_path)
            .json(&invalid_sensor)
            .expect_failure()
            .await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
    }
}
