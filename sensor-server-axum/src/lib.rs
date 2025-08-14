use axum::routing::MethodRouter;

pub mod api;
pub mod db;
pub mod middleware;
pub mod model;
pub mod schema;
pub mod sensor_server;

pub const PORT: u16 = 3000;

/// RoutePath struct
/// Represents a route in the SensorServer, just contains the path from the main 2025-08-13 17:57
/// I.e.: if the server is hosted in sensor_server.com and the route for users endpoint is
/// sensor_server.com/users, the RoutePath for users will contain a &str == "/users"
#[derive(Debug, Clone, Copy)]
pub struct RoutePath<'a>(&'a str);

impl<'a> RoutePath<'a> {
    pub fn as_str(&'a self) -> &'a str {
        self.0
    }

    pub fn from_str(s: &'a str) -> Option<RoutePath<'a>> {
        if !s.starts_with("/") {
            return None;
        }

        if s.ends_with("/") {
            return None;
        }

        Some(RoutePath(s))
    }
}

/// Requires implementors to provide a method for converting the implementing type into a resource
/// representation
pub trait ToRoute<'a> {
    fn to_route(&'a self) -> (RoutePath<'a>, &'a MethodRouter);
}

#[cfg(test)]
mod tests {}
