use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::{ParseError, Uuid};

use crate::{error::RefreshTokenErr, req::set_req_headers, LoginErr, RobinhoodErr};
use crate::{Robinhood, CLIENT_ID, EXPIRES_IN, LOG_IN_PATH, ROBINHOOD_API_URL, USER_AGENT};
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
    pub fn new(username: String, password: String) -> Self {
        let device_token = Uuid::new_v4();
        MfaLogin {
            username,
            password,
            device_token,
            user_agent: USER_AGENT.to_owned(),
            client_id: CLIENT_ID.to_owned(),
        }
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
    pub async fn request_mfa_code(&self) -> Result<(), RobinhoodErr> {
        let payload = self.build_login_payload();

        match set_req_headers(
            self,
            reqwest::Client::new().post(&format!("{}{}", ROBINHOOD_API_URL, LOG_IN_PATH)),
        )
        .json(&payload)
        .send()
        .await
        {
            Ok(res) => {
                match res.json::<Value>().await {
                    Ok(body) => {
                        if check_invalid_creds(&body) {
                            return Err(RobinhoodErr::InvalidCredentials);
                        }
                    }
                    Err(e) => {
                        return Err(RobinhoodErr::RequestError(e));
                    }
                };
            }
            Err(e) => {
                return Err(RobinhoodErr::RequestError(e));
            }
        }
        Ok(())
    }

    /// Logs in using an existing MFA code
    pub async fn log_in(self, mfa_code: String) -> Result<Robinhood, LoginErr> {
        let mut payload = match serde_json::to_value(self.build_login_payload()) {
            Ok(v) => v,
            Err(e) => {
                return Err(LoginErr::BadLoginBody(e.to_string()));
            }
        };
        // Add MFA code to the request payload
        match payload.as_object_mut() {
            Some(map) => {
                map.insert("mfa_code".to_owned(), Value::String(mfa_code));
                payload = serde_json::json!(map);
            }
            None => {
                return Err(LoginErr::EmptyLoginBody);
            }
        }
        // Make sure mfa_code is in the request body
        if let None = payload.get("mfa_code") {
            return Err(LoginErr::MissingMfaCode);
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
            Ok(v) => match v.json::<Value>().await {
                Ok(body) => {
                    if check_invalid_creds(&body) {
                        return Err(LoginErr::InvalidCredentials);
                    };
                    match serde_json::from_value(body) {
                        Ok(success) => success,
                        Err(e) => {
                            let msg = format!(
                                "Failed to serialize successful login response body: ({})",
                                e
                            );
                            return Err(LoginErr::BadResponseBody(msg));
                        }
                    }
                }
                Err(e) => {
                    return Err(LoginErr::RequestError(e));
                }
            },
            Err(e) => {
                return Err(LoginErr::RequestError(e));
            }
        };
        // Build a Robinhood session
        Ok(Robinhood {
            device_token: self.device_token,
            password: Some(self.password),
            username: Some(self.username),
            user_agent: self.user_agent,
            token: login_response.access_token,
            refresh_token: login_response.refresh_token,
            token_expires_in: login_response.expires_in,
            auto_refresh: true,
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
    pub fn change_device_token_str(&mut self, device_token: String) -> Result<(), ParseError> {
        match Uuid::from_str(&device_token) {
            Ok(new_device_token) => {
                self.device_token = new_device_token;
                Ok(())
            }
            Err(e) => {
                return Err(e);
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

pub struct NewToken {
    pub token: String,
    pub refresh_token: String,
}

impl Robinhood {
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
        let mfa_client = MfaLogin::new(username, password);
        mfa_client.request_mfa_code().await?;
        Ok(mfa_client)
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
    pub async fn token_login(
        token: String,
        refresh_token: String,
        device_token: Uuid,
    ) -> Robinhood {
        Robinhood {
            device_token,
            password: None,
            username: None,
            user_agent: USER_AGENT.to_owned(),
            token,
            refresh_token,
            token_expires_in: EXPIRES_IN,
            auto_refresh: true,
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

    /// Change username and password
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = Some(username);
        self.password = Some(password);
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
    pub fn change_device_token_str(&mut self, device_token: String) -> Result<(), ParseError> {
        match Uuid::from_str(&device_token) {
            Ok(new_device_token) => {
                self.device_token = new_device_token;
                Ok(())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    pub fn set_refresh_token(&mut self, refresh_token: String) {
        self.refresh_token = refresh_token;
    }

    pub fn get_device_token(&self) -> Uuid {
        self.device_token
    }

    pub fn get_refresh_token(&self) -> String {
        self.refresh_token.clone()
    }

    pub fn get_token(&self) -> String {
        self.token.clone()
    }

    /// Default is `true`
    ///
    /// The lib will try to refresh the token if any of the calls return a 401
    ///
    /// This setting enables the auto refresh of a token if a call returns a 401
    pub fn set_auto_refresh(&mut self, auto_refresh: bool) {
        self.auto_refresh = auto_refresh;
    }

    // Necessary after every 24h since access_token has an expiration of 24h
    pub async fn refresh_token(
        &mut self,
        old_refresh_token: Option<String>,
    ) -> Result<Option<NewToken>, RefreshTokenErr> {
        // Make sure there is no data race when updating the token
        if let Some(old_token) = old_refresh_token {
            if self.refresh_token != old_token {
                return Ok(None);
            };
        }

        let req_token_payload = RefreshTokenPayload {
            client_id: CLIENT_ID.to_owned(),
            device_token: self.device_token,
            grant_type: GrantType::RefreshToken,
            refresh_token: self.refresh_token.clone(),
            scope: Scope::Internal,
            token_type: TokenType::Bearer,
        };
        let req = reqwest::Client::new().post(&format!("{}{}", ROBINHOOD_API_URL, LOG_IN_PATH));
        let login_response: LoginSuccess = match set_req_headers(self, req)
            .json(&req_token_payload)
            .send()
            .await
        {
            Ok(v) => match v.json::<Value>().await {
                Ok(body) => {
                    // Check if refresh_token was invalid
                    if let Some(err_msg) = body["error"].as_str() {
                        if err_msg == "invalid_grant" {
                            return Err(RefreshTokenErr::BadRefreshToken(
                                self.refresh_token.clone(),
                            ));
                        }
                    }
                    match serde_json::from_value::<LoginSuccess>(body) {
                        Ok(success) => success,
                        Err(e) => {
                            let msg = format!(
                                "Failed to serialize successful login response body: ({})",
                                e
                            );
                            return Err(RefreshTokenErr::WrongResponseBody(msg));
                        }
                    }
                }
                Err(e) => return Err(RefreshTokenErr::RequestError(e)),
            },
            Err(e) => return Err(RefreshTokenErr::RequestError(e)),
        };
        self.refresh_token = login_response.refresh_token;
        self.token = login_response.access_token;
        self.token_expires_in = login_response.expires_in;
        Ok(Some(NewToken {
            token: self.token.clone(),
            refresh_token: self.refresh_token.clone(),
        }))
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

pub fn check_invalid_creds(body: &Value) -> bool {
    if let Some(detail_msg) = body["detail"].as_str() {
        if detail_msg.contains("Unable to log in with provided credentials") {
            return true;
        }
    }
    return false;
}
