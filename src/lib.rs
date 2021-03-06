//! # Robinhood
//!
//! A library wrapping Robinhood unofficial API
//!
//! # Example
//!
//! ```
//! use robinhood;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let username = "my_username".to_owned();
//!     let password = "password".to_owned()
//!     let mfa_client = robinhood::mfa_login(username, password).await?;
//!     // By this point you should have received an SMS/E-mail containing a login code
//!     // Add your own logic to wait for the code and insert it in the next function
//!     // You could have a loop trying to retrieve it from a database or if this is run as a script from std::input
//!     let mfa_code = ...
//!     // Needs to be `mut` will revise this in the future
//!     let mut robinhood_client = mfa_client.log_in(mfa_code).await?;
//!
//!     // Get the price of SPY in an interval
//!     use std::time::Duration;
//!     use std::thread;
//!
//!     loop {
//!         // Use some timer to not spam Robinhood with requests.. you might get banned
//!         thread::sleep(Duration::from_millis(500));
//!         let price: usize = robinhood_client.get_price("SPY").await?;
//!         println!("{}", price);
//!     }
//!
//! }
//! ```
use error::RobinhoodErr;
pub use reqwest::Error as ReqwestError;

use login::MfaLogin;
use uuid::Uuid;

// Base URL
const ROBINHOOD_API_URL: &str = "https://api.robinhood.com/";
// Paths
const LOG_IN_PATH: &str = "oauth2/token/";
const QUOTES_PATH: &str = "quotes/";

const CLIENT_ID: &str = "c82SH0WZOsabOXGP2sxqcj34FxkvfnWRZBKlBjFS";
const EXPIRES_IN: u32 = 86400;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36 Edg/88.0.705.81";

pub mod error;
mod login;
mod queries;
mod req;

/// A Robinhood client instance
pub struct Robinhood {
    username: Option<String>,
    password: Option<String>,
    token_expires_in: u32,
    token: String,
    refresh_token: String,
    device_token: Uuid,
    user_agent: String,
    auto_refresh: bool,
    retries: usize,
}
/// Initializes an MFA login session
///
/// # Example
///
/// ```
/// use robinhood;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let username = "my_username".to_owned();
///     let password = "password".to_owned()
///     let mfa_client = robinhood::mfa_login(username, password).await?;
///     // By this point you should have received an SMS/E-mail containing a login code
///     // Add your own logic to wait for the code and insert it in the next function
///     // You could have a loop trying to retrieve it from a database or if this is run as a script from std::input
///     let mfa_code = ...
///     // Needs to be `mut` will revise this in the future
///     let mut robinhood_client = mfa_client.log_in(mfa_code).await?;
///
///     // Get the price of SPY in an interval
///     use std::time::Duration;
///     use std::thread;
///
///     loop {
///         // Use some timer to not spam Robinhood with requests.. you might get banned
///         thread::sleep(Duration::from_millis(500));
///         let price: usize = robinhood_client.get_price("SPY").await?;
///         println!("{}", price);
///     }
///
/// }
/// ```
pub async fn mfa_login(username: String, password: String) -> Result<MfaLogin, RobinhoodErr> {
    Robinhood::mfa_login(username, password).await
}

/// If you already have a token and a refresh_token then use this to instantiate
/// the session
///
/// # Example
///
/// ```
/// use robinhood;
/// use uuid;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // If you are choosing this method then you probably logged in before using the mfa approach
///     let token = "abc123".to_owned();
///     let refresh_token = "123abz".to_owned()
///     let device_token = uuid::Uuid::new_v4()
///     let mfa_client = robinhood::token_login(token, refresh_token, device_token).await?;
///     // Any calls than requires authentication will fail if the token is expired.
///     // If you have a valid refresh token
///     // and `Robinhood.auto_refresh` is set to true then it will create a new one.
///
///     // Get the price of SPY in an interval
///     use std::time::Duration;
///     use std::thread;
///
///     loop {
///         // Use some timer to not spam Robinhood with requests.. you might get banned
///         thread::sleep(Duration::from_millis(500));
///         let price: usize = robinhood_client.get_price("SPY").await?;
///         println!("{}", price);
///     }
///
/// }
/// ```
pub async fn token_login(token: String, refresh_token: String, device_token: Uuid) -> Robinhood {
    Robinhood::token_login(token, refresh_token, device_token).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
