mod appointments;
mod auth;
mod clinic;
mod finance;
mod stock;
mod user;

pub use appointments::*;
pub use auth::*;
pub use clinic::*;
pub use finance::*;
pub use stock::*;
pub use user::*;

pub const API_BASE: &str = match option_env!("API_BASE_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:4000/api",
};
