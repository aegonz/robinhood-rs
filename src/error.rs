use std::num::ParseFloatError;

use crate::ReqwestError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RobinhoodErr {
    #[error("Unauthorized request 401")]
    Unauthorized,
    #[error("404 NOT FOUND URL: {0}")]
    NotFound(String),
    #[error("{0}")]
    NetworkError(String),
    #[error("{0}")]
    RequestError(#[from] ReqwestError),
    /// Invalid log in credentials
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::RobinhoodErr;
    /// let price = robinhood.get_price("SPY").await {
    ///     Ok(price) => price,
    ///     Err(e) => {
    ///         match e {
    ///            RobinhoodErr::ParseFloatError => {
    ///                 panic!("Expected string '420.69' as f32 got different value")
    ///            },
    ///            _ => {panic!(e)}
    ///         }
    ///     }
    /// };
    /// ```
    #[error("{0}")]
    ParseFloatError(#[from] ParseFloatError),
    /// Invalid log in credentials
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::RobinhoodErr;
    /// let mut robinhood_client = robinhood::mfa_login(username, password).await {
    ///     Ok(client) => client,
    ///     Err(e) => {
    ///         match e {
    ///            RobinhoodErr::InvalidCredentials => {
    ///                 panic!("wrong username/password")
    ///            },
    ///            _ => {panic!(e)}
    ///         }
    ///     }
    /// };
    /// ```
    #[error("Invalid username/password")]
    InvalidCredentials,
    #[error("{0}")]
    BadResponseBody(String),
    #[error("The refresh token '{0}' is no longer valid")]
    BadRefreshToken(String),
}

#[derive(Error, Debug)]
pub enum LoginErr {
    #[error("{0}")]
    RequestError(#[from] ReqwestError),
    #[error("Failed to serialize login payload ({0})")]
    BadLoginBody(String),
    #[error("Log in payload is empty. This should never happen. Something went terrible wrong")]
    EmptyLoginBody,
    #[error("Mfa code was not added to the request body correctly")]
    MissingMfaCode,
    /// Invalid log in credentials
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::LoginErr;
    /// let mut robinhood_client = mfa_client.log_in(mfa_code).await {
    ///     Ok(client) => client,
    ///     Err(e) => {
    ///         match e {
    ///            LoginErr::InvalidCredentials => {
    ///                 panic!("wrong username/password")
    ///            },
    ///            _ => {panic!(e)}
    ///         }
    ///     }
    /// };
    /// ```
    #[error("Invalid username/password")]
    InvalidCredentials,
    #[error("{0}")]
    BadResponseBody(String),
}
