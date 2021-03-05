use crate::Result;
use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::req::{ReqKind, RobinhoodReq};
use crate::{Robinhood, QUOTES_PATH, ROBINHOOD_API_URL};

impl Robinhood {
    pub async fn get_quote(&mut self, symbol: String) -> Result<QuotesResponse> {
        let url = &format!("{}{}{}", ROBINHOOD_API_URL, QUOTES_PATH, symbol);

        let response = self
            .req(RobinhoodReq {
                kind: ReqKind::Get,
                payload: None,
                url,
            })
            .await?;

        match response.json::<QuotesResponse>().await {
            Ok(res) => return Ok(res),
            Err(e) => {
                bail!(e)
            }
        };
    }

    pub async fn get_price(&mut self, symbol: String) -> Result<usize> {
        let hu = self.get_quote(symbol).await?;
        match hu.last_trade_price.parse::<usize>() {
            Ok(v) => Ok(v),
            Err(e) => {
                bail!(e)
            }
        }
    }
}

// "ask_price": "394.750000",
// "ask_size": 30,
// "bid_price": "371.000000",
// "bid_size": 100,
// "last_trade_price": "381.420000",
// "last_extended_hours_trade_price": "380.910000",
// "previous_close": "386.560000",
// "adjusted_previous_close": "386.540000",
// "previous_close_date": "2021-03-02",
// "symbol": "SPY",
// "trading_halted": false,
// "has_traded": true,
// "last_trade_price_source": "consolidated",
// "updated_at": "2021-03-04T01:00:00Z",
// "instrument": "https://api.robinhood.com/instruments/8f92e76f-1e0e-4478-8580-16a6ffcfaef5/",
// "instrument_id": "8f92e76f-1e0e-4478-8580-16a6ffcfaef5"
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QuotesResponse {
    ask_price: String,
    ask_size: usize,
    bid_price: String,
    bid_size: usize,
    last_trade_price: String,
    last_extended_hours_trade_price: Option<String>,
    previous_close: String,
    adjusted_previous_close: String,
    previous_close_date: String,
    symbol: String,
    trading_halted: bool,
    has_traded: bool,
    last_trade_price_source: String,
    updated_at: String,
    instrument: String,
    instrument_id: String,
}
