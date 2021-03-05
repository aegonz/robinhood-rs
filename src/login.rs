use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::req::set_req_headers;
use crate::{Result, Robinhood, CLIENT_ID, EXPIRES_IN, LOG_IN_PATH, ROBINHOOD_API_URL, USER_AGENT};
pub trait AgentToken {
    fn get_user_agent(&self) -> &str;
    fn get_token(&self) -> Option<&str>;
}

// client_id: "c82SH0WZOsabOXGP2sxqcj34FxkvfnWRZBKlBjFS",
// device_token: "Uuid",
// expires_in: 86400,
// grant_type: "password",
// scope: "internal",
// username: "<>",
// password: "<>"
// mfa_code: "111111"
#[derive(Debug, Serialize, Deserialize)]
pub struct LogInPayload {
    client_id: String,
    device_token: Uuid,
    expires_in: u32,
    grant_type: GrantType,
    scope: Scope,
    username: String,
    password: String,
}

// token_type: "Bearer",
// scope: "internal",
// refresh_token: "<>",
// "grant_type": "refresh_token",
// "client_id": "<>",
// "device_token": "<>"
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    token_type: TokenType,
    scope: Scope,
    refresh_token: String,
    grant_type: GrantType,
    client_id: String,
    device_token: Uuid,
}

// access_token: "<>",
// expires_in: 740067,
// token_type: "Bearer",
// scope: "internal",
// refresh_token: "<>",
// mfa_code: "329503",
// backup_code: null
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LoginSuccess {
    access_token: String,
    expires_in: u32,
    token_type: TokenType,
    scope: Scope,
    refresh_token: String,
    mfa_code: Option<String>,
    backup_code: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum GrantType {
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Scope {
    #[serde(rename = "internal")]
    Internal,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Internal
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TokenType {
    Bearer,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Bearer
    }
}

pub struct MfaLogin {
    username: String,
    password: String,
    device_token: Uuid,
    user_agent: String,
    client_id: String,
}

impl MfaLogin {
    /// Instantiates a new MfaLogin client
    async fn new(username: String, password: String) -> Result<Self> {
        let device_token = Uuid::new_v4();
        Ok(MfaLogin {
            username,
            password,
            device_token,
            user_agent: USER_AGENT.to_owned(),
            client_id: CLIENT_ID.to_owned(),
        })
    }

    /// Build a log in payload based on struct values
    fn build_login_payload(&self) -> LogInPayload {
        LogInPayload {
            client_id: self.client_id.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            expires_in: EXPIRES_IN,
            scope: Scope::Internal,
            grant_type: GrantType::Password,
            device_token: self.device_token,
        }
    }

    /// Logs into Robinhood in order to request an MFA code (SMS, E-Mail)
    pub async fn request_mfa_code(&self) -> Result<()> {
        let payload = self.build_login_payload();

        if let Err(e) = set_req_headers(
            self,
            reqwest::Client::new().post(&format!("{}{}", ROBINHOOD_API_URL, LOG_IN_PATH)),
        )
        .json(&payload)
        .send()
        .await
        {
            bail!(format!("Failed to log in ({})", e));
        };
        Ok(())
    }

    /// Logs in using an existing MFA code
    pub async fn log_in(self, mfa_code: String) -> Result<Robinhood> {
        let mut payload = match serde_json::to_value(self.build_login_payload()) {
            Ok(v) => v,
            Err(e) => {
                bail!(format!("Failed to serialize login payload ({})", e))
            }
        };
        // Add MFA code to the request payload
        match payload.as_object_mut() {
            Some(map) => {
                map.insert("mfa_code".to_owned(), Value::String(mfa_code));
            }
            None => {
                bail!("Failed to add mfa_code to the payload. This should have never happened")
            }
        }
        // Send request to Robinhood
        let login_response: LoginSuccess = match set_req_headers(
            &self,
            reqwest::Client::new().post(&format!("{}{}", ROBINHOOD_API_URL, LOG_IN_PATH)),
        )
        .json(&payload)
        .send()
        .await
        {
            Ok(v) => match v.json().await {
                Ok(v) => v,
                Err(e) => {
                    bail!(format!("Failed to serialize the response body ({})", e))
                }
            },
            Err(e) => {
                bail!(format!("Failed to log in ({})", e));
            }
        };
        // Build a Robinhood session
        Ok(Robinhood {
            device_token: self.device_token,
            password: self.password,
            username: self.username,
            user_agent: self.user_agent,
            token: login_response.access_token,
            refresh_token: login_response.refresh_token,
            token_expires_in: login_response.expires_in,
            retries: 200,
            auto_refresh: true,
            auto_retry: true,
        })
    }

    /// Change username and password
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = username;
        self.password = password;
    }

    /// Change device token
    ///
    /// Device token should be unique based on each device (Phone, Browser, etc..).
    /// This can be any Uuid
    ///
    /// The login session will be tied to this specific device.
    ///
    /// You should only have the need to use this if you logged in with a different device ID
    /// and want to use the same session tied to that device
    pub fn change_device_token(&mut self, device_token: Uuid) {
        self.device_token = device_token;
    }

    /// Change device token
    ///
    /// Device token should be unique based on each device (Phone, Browser, etc..).
    /// This can be any Uuid
    ///
    /// The login session will be tied to this specific device.
    ///
    /// You should only have the need to use this if you logged in with a different device ID
    /// and want to use the same session tied to that device
    pub fn change_device_token_str(&mut self, device_token: String) -> Result<()> {
        match Uuid::from_str(&device_token) {
            Ok(new_device_token) => {
                self.device_token = new_device_token;
                Ok(())
            }
            Err(_) => {
                bail!("New device token is not a valid UUID")
            }
        }
    }

    /// The default agent is
    /// `"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36 Edg/88.0.705.81"`
    ///
    /// This is an Edge browser `Edg/88.0.705.81`
    ///
    /// Change it to whatever you like.
    pub fn change_agent(&mut self, user_agent: String) {
        self.user_agent = user_agent;
    }
}

impl AgentToken for MfaLogin {
    fn get_user_agent(&self) -> &str {
        &self.user_agent
    }

    fn get_token(&self) -> Option<&str> {
        None
    }
}

type Token = String;
type RefreshToken = String;

impl Robinhood {
    /// Initializes an MFA login session
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::Robinhood;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mfa_client = Robinhood::mfa_login("my_username".to_owned(), "password".to_owned());
    ///     mfa_client.request_mfa_code().await.unwrap();
    ///     // By this point you should have received an SMS/E-mail containing a login code
    ///     // Add your own logic to wait for the code and insert it in the next function
    ///     // You could have a loop trying to retrieve it from a database or if this is run as a script from std::input
    ///     let mfa_code = ...
    ///     let robinhood = mfa_client.log_in(mfa_code).await.unwrap();
    ///
    ///     // Get the price of SPY in an interval
    ///     use std::time::Duration;
    ///     use std::thread;
    ///
    ///     loop {
    ///         // Use some timer to not spam Robinhood with request.. you might get banned
    ///         thread::sleep(Duration::from_millis(500));
    ///         let price: usize = robinhood.get_price("SPY").await.unwrap().last_trade_price;
    ///         println!("{}", price);
    ///     }
    ///
    /// }
    /// ```
    pub async fn mfa_login(username: String, password: String) -> Result<MfaLogin> {
        Ok(MfaLogin::new(username, password).await?)
    }

    /// The default agent is
    /// `"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36 Edg/88.0.705.81"`
    ///
    /// This is an Edge browser `Edg/88.0.705.81`
    ///
    /// Change it to whatever you like.
    pub fn change_agent(&mut self, user_agent: String) {
        self.user_agent = user_agent;
    }

    /// Change username and password
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = username;
        self.password = password;
    }

    /// Change device token
    ///
    /// Device token should be unique based on each device (Phone, Browser, etc..).
    /// This can be any Uuid
    ///
    /// The login session will be tied to this specific device.
    ///
    /// You should only have the need to use this if you logged in with a different device ID
    /// and want to use the same session tied to that device
    pub fn change_device_token(&mut self, device_token: Uuid) {
        self.device_token = device_token;
    }

    /// Change device token
    ///
    /// Device token should be unique based on each device (Phone, Browser, etc..).
    /// This can be any Uuid
    ///
    /// The login session will be tied to this specific device.
    ///
    /// You should only have the need to use this if you logged in with a different device ID
    /// and want to use the same session tied to that device
    pub fn change_device_token_str(&mut self, device_token: String) -> Result<()> {
        match Uuid::from_str(&device_token) {
            Ok(new_device_token) => {
                self.device_token = new_device_token;
                Ok(())
            }
            Err(_) => {
                bail!("New device token is not a valid UUID")
            }
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    pub fn set_refreshtoken(&mut self, refresh_token: String) {
        self.refresh_token = refresh_token;
    }

    // Necessary after every 24h since access_token has an expiration of 24h
    pub async fn refresh_token(&mut self) -> Result<(Token, RefreshToken)> {
        let new_token_payload = refresh_token(
            self,
            &RefreshTokenPayload {
                client_id: CLIENT_ID.to_owned(),
                device_token: self.device_token,
                grant_type: GrantType::RefreshToken,
                refresh_token: self.refresh_token.clone(),
                scope: Scope::Internal,
                token_type: TokenType::Bearer,
            },
        )
        .await?;
        self.refresh_token = new_token_payload.refresh_token;
        self.token = new_token_payload.access_token;
        self.token_expires_in = new_token_payload.expires_in;
        Ok((self.token.clone(), self.refresh_token.clone()))
    }
}

impl AgentToken for Robinhood {
    fn get_user_agent(&self) -> &str {
        &self.user_agent
    }

    fn get_token(&self) -> Option<&str> {
        Some(&self.token)
    }
}

pub async fn refresh_token<T: AgentToken>(
    requestor: &T,
    payload: &RefreshTokenPayload,
) -> Result<LoginSuccess> {
    match payload.grant_type {
        GrantType::RefreshToken => {
            let req = reqwest::Client::new().post(&format!("{}{}", ROBINHOOD_API_URL, LOG_IN_PATH));
            let login_response: LoginSuccess =
                match set_req_headers(requestor, req).json(&payload).send().await {
                    Ok(v) => match v.json().await {
                        Ok(res) => res,
                        Err(e) => {
                            bail!("Failed to serialize token refresh response ({})", e)
                        }
                    },
                    Err(e) => {
                        bail!("Failed to refresh token ({})", e)
                    }
                };
            Ok(login_response)
        }
        _ => {
            bail!(format!(
                "Wrong grant type. Expected GrantType::RefreshToken got {:?}",
                payload.grant_type
            ))
        }
    }
}
