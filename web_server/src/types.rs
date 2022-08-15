use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

// Generic error for other stuff. Implementing From on Error would cause everything to have generic messages and kind
// so this is here to clearly distinguish between generic errors and errors with good messages
#[derive(Debug)]
pub struct AnyError(String);

impl<T: Display> From<T> for AnyError {
    fn from(error: T) -> Self {
        AnyError(error.to_string())
    }
}

// Error type that emits good HTTP status
#[derive(Debug)]
pub enum Error {
    Generic(String),
    NotFound(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        warn_!("Error: {:?}", self);
        match self {
            Error::Generic(_) => Err(rocket::http::Status::InternalServerError),
            Error::NotFound(_) => Err(rocket::http::Status::NotFound),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// ETFChart is the structure that contains all the information used to draw the historical holding breakdown chart
#[derive(Serialize, Debug, Clone)]
pub struct ETFChart {
    pub etf_ticker: String,
    pub etf_name: String,
    pub holding_details: HashMap<String, ETFChartHoldingDetails>,
    pub price_chart: Vec<ETFChartPrice>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ETFChartHoldingDetails {
    pub ticker: String,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ETFChartPrice {
    pub timestamp: i64,
    pub etf_price: Option<f64>,
    pub holding_prices: HashMap<String, f64>,
}

// ETFDetails (and EquityDetails) format all the available information into an easy to use
// structure
#[derive(Serialize, Debug, Clone)]
pub struct ETFDetails {
    pub ticker: String,
    pub name: String,
    pub equity_holdings: Vec<EquityDetails>,
    pub other_holdings: HashMap<String, f64>,
    pub prices: Option<Vec<HistoricalPrices>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EquityDetails {
    pub ticker: String,
    pub name: String,
    pub weight: f64,
    pub location: String,
    pub exchange: String,
    pub prices: Option<Vec<HistoricalPrices>>,
}

// Return type from yahoo fetch_historical_prices
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
