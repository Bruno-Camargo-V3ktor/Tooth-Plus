mod appointments;
mod auth;
mod clinic;
pub mod documents;
pub mod finance;
mod patients;
mod stock;
pub mod treatments;
mod user;

pub use appointments::*;
pub use auth::*;
pub use clinic::*;
pub use documents::*;
pub use finance::*;
pub use patients::*;
pub use stock::*;
pub use treatments::*;
pub use user::*;

pub const API_BASE: &str = match option_env!("API_BASE_URL") {
    Some(url) => url,
    None => "https://toothplus-api.bluerosestudio.com.br/api",
};
