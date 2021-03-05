pub use anyhow::{Error, Result};

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
