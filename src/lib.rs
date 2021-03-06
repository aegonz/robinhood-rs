pub use anyhow::{Error, Result};

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

mod login;
mod queries;
mod req;

/// A library wrapping Robinhood unofficial API
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
pub struct Robinhood {
    username: Option<String>,
    password: Option<String>,
    token_expires_in: u32,
    token: String,
    refresh_token: String,
    device_token: Uuid,
    user_agent: String,
    auto_refresh: bool,
    auto_retry: bool,
    retries: usize,
}

pub async fn mfa_login(username: String, password: String) -> Result<MfaLogin> {
    Robinhood::mfa_login(username, password).await
}

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
