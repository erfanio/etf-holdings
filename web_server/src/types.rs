//! Contains response types, customer errors, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Error type that emits good HTTP status by implementing rocket::response::Responder.
#[derive(Debug)]
pub enum GoodError {
    Generic(String),
    NotFound(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for GoodError {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        warn_!("Error: {:?}", self);
        match self {
            GoodError::Generic(_) => Err(rocket::http::Status::InternalServerError),
            GoodError::NotFound(_) => Err(rocket::http::Status::NotFound),
        }
    }
}

/// Convert to GoodError from anything that implements Display
///
/// Implementing From<Display> on GoodError would cause everything to have generic messages and kind
pub fn to_good_error<T: Display>(error: T) -> GoodError {
    GoodError::Generic(error.to_string())
}

/// Result with a GoodError to produce good HTTP status
pub type GoodResult<T> = std::result::Result<T, GoodError>;

/// Response type for chart endpoint.
#[derive(Serialize, Debug, Clone)]
pub struct ChartResponse {
    pub etf_ticker: String,
    pub etf_name: String,
    pub holding_details: HashMap<String, ChartHoldingDetails>,
    pub chart: Vec<ChartPrice>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChartHoldingDetails {
    pub ticker: String,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChartPrice {
    pub timestamp: i64,
    pub etf_price: f64,
    pub holding_prices: HashMap<String, f64>,
}

/// Response type for details endpoint.
#[derive(Serialize, Debug, Clone)]
pub struct DetailsResponse {
    pub ticker: String,
    pub name: String,
    pub equity_holdings: Vec<DetailsEquityHolding>,
    pub other_holdings: HashMap<String, f64>,
    pub prices: Option<Vec<HistoricalPrices>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DetailsEquityHolding {
    pub ticker: String,
    pub name: String,
    pub weight: f64,
    pub location: String,
    pub exchange: String,
    pub prices: Option<Vec<HistoricalPrices>>,
}

/// Return type for yahoo::fetch_historical_prices, but it's also used in other places
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoricalPrices {
    pub timestamp: i64,
    pub volume: i64,
    pub open: f64,
    pub low: f64,
    pub high: f64,
    pub close: f64,
    pub adjclose: f64,
}
