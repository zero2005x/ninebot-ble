extern crate uuid;

pub mod clone_connection;
pub mod consts;
pub mod mi_crypto;
pub mod protocol;
pub mod session;

pub use clone_connection::ScooterConnection;

mod connection;
mod login;
mod register;
mod scanner;

pub use connection::ConnectionHelper;
pub use login::LoginRequest;
pub use mi_crypto::AuthToken;
pub use register::RegistrationError;
pub use register::RegistrationRequest;
pub use scanner::ScannerError;
pub use scanner::ScannerEvent;
pub use scanner::ScooterScanner;
pub use scanner::TrackedDevice;

pub use session::{BatteryInfo, GeneralInfo, MiSession, MotorInfo, Payload, TailLight};
