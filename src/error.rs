use std::num::ParseIntError;

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
    #[error("ERROR: <_> -[ A required task with ID {0} is missing from the workflow ]-")]
    MissingTask(String),
    #[error("{0}")]
    RequestError(#[from] ReqwestError),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("{0}")]
    Internal(String),
    /// Invalid log in credentials
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::RobinhoodErr;
    ///
    /// ...
    /// 
    /// let mut robinhood_client = robinhood::mfa_login(username, password).await {
    ///     Ok(client) => client,
    ///     Err(e) => {
    ///         match e {
    ///            RobinhoodErr::InvalidCredentials => {
    ///                 panic!("oops wrong username/password")
    ///            },
    ///            _ => {panic!(e)}
    ///         }
    ///     }
    /// };
    /// ```
    #[error("Invalid username/password")]
    InvalidCredentials,
}
