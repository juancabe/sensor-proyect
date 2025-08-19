use axum::routing::MethodRouter;

pub mod api;
pub mod auth;
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
#[derive(Debug, Clone)]
pub struct RoutePath(String);

impl RoutePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_string(s: String) -> Option<RoutePath> {
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
pub trait ToRoute {
    fn to_route(self) -> (RoutePath, MethodRouter);
}

#[cfg(test)]
mod tests {}
