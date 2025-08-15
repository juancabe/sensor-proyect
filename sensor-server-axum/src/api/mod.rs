pub mod endpoints;
pub mod route;
pub mod types;

use crate::api::route::Route;

pub trait Endpoint {
    fn routes(&self) -> &[Route];
}
