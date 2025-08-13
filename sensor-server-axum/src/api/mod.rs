pub mod endpoints;
pub mod route;

use crate::api::route::Route;

pub trait Endpoint {
    fn routes(&self) -> &[Route];
}
