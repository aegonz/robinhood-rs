use reqwest::{RequestBuilder, Response};
use serde_json::Value;

use crate::{error::RobinhoodErr, login::AgentToken, Robinhood};

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
    pub async fn req(&mut self, request: RobinhoodReq<'_>) -> Result<Response, RobinhoodErr> {
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

    async fn send_req(&mut self, req: RequestBuilder) -> Result<Response, RobinhoodErr> {
        match req.send().await {
            Ok(res) => return Ok(res),
            Err(e) => {
                if let Some(status_code) = e.status() {
                    // If status code is a 401 try to refresh the token
                    if status_code.as_u16() == 401 && self.auto_refresh {
                        if let Err(_) = self.refresh_token().await {
                            return Err(RobinhoodErr::Unauthorized);
                        }
                    }
                    if status_code == 404 {
                        match e.url() {
                            Some(url) => {
                                return Err(RobinhoodErr::NotFound(url.to_string()));
                            }
                            None => {}
                        }
                    }
                    return Err(RobinhoodErr::Unauthorized);
                }
                return Err(RobinhoodErr::RequestError(e));
            }
        }
    }
}
