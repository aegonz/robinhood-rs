use crate::error::RobinhoodErr;
use serde::{Deserialize, Serialize};

use crate::req::{ReqKind, RobinhoodReq};
use crate::{Robinhood, QUOTES_PATH, ROBINHOOD_API_URL};

impl Robinhood {
    /// Calls api.robinhood.com/quotes/(symbol)/ and returns the body as `QuotesResponse`
    pub async fn get_quote(&mut self, symbol: String) -> Result<QuotesResponse, RobinhoodErr> {
        let url = &format!("{}{}{}/", ROBINHOOD_API_URL, QUOTES_PATH, symbol);
        let response = self
            .req(RobinhoodReq {
                kind: ReqKind::Get,
                payload: None,
                url,
            })
            .await?;
        match response.json::<QuotesResponse>().await {
            Ok(res) => return Ok(res),
            Err(e) => return Err(RobinhoodErr::RequestError(e)),
        };
    }

    /// Calls api.robinhood.com/quotes/(symbol)/ to retrieve a `QuotesResponse`
    /// and extracts the `last_trade_price` from the body
    pub async fn get_price(&mut self, symbol: String) -> Result<usize, RobinhoodErr> {
        let quote = self.get_quote(symbol).await?;
        match quote.last_trade_price.parse::<usize>() {
            Ok(v) => Ok(v),
            Err(e) => return Err(RobinhoodErr::ParseIntError(e)),
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
    pub ask_price: String,
    pub ask_size: usize,
    pub bid_price: String,
    pub bid_size: usize,
    pub last_trade_price: String,
    pub last_extended_hours_trade_price: Option<String>,
    pub previous_close: String,
    pub adjusted_previous_close: String,
    pub previous_close_date: String,
    pub symbol: String,
    pub trading_halted: bool,
    pub has_traded: bool,
    pub last_trade_price_source: String,
    pub updated_at: String,
    pub instrument: String,
    pub instrument_id: String,
}
