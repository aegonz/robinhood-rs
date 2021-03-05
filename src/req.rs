use std::{thread, time::Duration};

use anyhow::bail;
use reqwest::{RequestBuilder, Response};
use serde_json::Value;

use crate::{login::AgentToken, Result, Robinhood};

pub fn set_req_headers<T: AgentToken>(requestor: &T, req: RequestBuilder) -> RequestBuilder {
    let mut rb_req = req.header("User-Agent", requestor.get_user_agent());
    if let Some(token) = requestor.get_token() {
        rb_req = rb_req.header("Authorization", format!("Bearer {}", token));
    }
    rb_req
}

pub enum ReqKind {
    Post,
    Get,
}

pub struct RobinhoodReq<'a> {
    pub kind: ReqKind,
    pub url: &'a str,
    pub payload: Option<&'a Value>,
}

impl Robinhood {
    pub async fn req(&mut self, request: RobinhoodReq<'_>) -> Result<Response> {
        match request.kind {
            ReqKind::Post => {
                let mut req = set_req_headers(self, reqwest::Client::new().post(request.url));
                if let Some(payload) = request.payload {
                    req = req.json(payload)
                }
                self.send_req(req).await
            }
            ReqKind::Get => {
                let req = set_req_headers(self, reqwest::Client::new().get(request.url));
                self.send_req(req).await
            }
        }
    }

    async fn send_req(&mut self, req: RequestBuilder) -> Result<Response> {
        loop {
            let req = match req.try_clone() {
                Some(rq) => rq,
                None => {
                    bail!("Failed to clone request. Might be a stream")
                }
            };
            match req.send().await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if let Some(status_code) = e.status() {
                        // If status code is a 401 try to refresh the token
                        if status_code.as_u16() == 401 && self.auto_refresh {
                            if let Err(e) = self.refresh_token().await {
                                bail!(e);
                            }
                        // If Robinhood is unreachable retry the request
                        } else if status_code.is_server_error() && self.auto_retry {
                            println!("Server error {} with status code {}", e, status_code);
                            thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                        bail!(e);
                    }

                    if (e.is_connect() || e.is_timeout()) && self.auto_retry {
                        println!("Connection error {}", e);
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    bail!(e);
                }
            }
        }
    }

    /// Default is `true`
    ///
    /// The lib will try to refresh the token if any of the calls return a 401
    ///
    /// This setting enables the auto refresh of a token if a call returns a 401
    pub fn set_auto_refresh(&mut self, retries: usize) {
        self.retries = retries;
    }

    /// Default retries is 200
    ///
    /// The lib will try to refresh the token if any of the calls return a 401
    ///
    /// If the auto refresh failed with a connection error it will retry.
    ///
    /// Other types of connection error should be handled application specific instead
    pub fn set_retries(&mut self, retries: usize) {
        self.retries = retries;
    }
}
