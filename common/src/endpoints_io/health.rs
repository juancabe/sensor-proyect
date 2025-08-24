#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "./api/endpoints/health/")]
pub struct GetHealth {}
