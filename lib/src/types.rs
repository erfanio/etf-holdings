//! Contains different common types, structs, errors...

use async_trait::async_trait;
use serde::Serialize;
use std::fmt::Display;

/// ETF details including holding information
#[derive(Serialize, Debug, Clone)]
pub struct ETF {
    pub ticker: String,
    pub name: String,
    pub last_update: String,
    pub outstanding_shares: f64,
    pub holdings: Vec<Holding>,
}

/// ETF Holding details
#[derive(Serialize, Debug, Clone)]
pub struct Holding {
    pub ticker: String,
    pub name: String,
    pub asset_class: String,
    pub market_value: f64,
    pub weight: f64,
    pub notional_value: f64,
    pub shares: f64,
    pub price: f64,
    pub location: String,
    pub exchange: String,
    pub currency: String,
    pub fx_rate: f64,
    pub market_currency: String,
}

/// Limited ETF information used for listing available ETFs
#[derive(Serialize, Debug, Clone)]
pub struct ETFListItem {
    pub ticker: String,
    pub name: String,
}

/// Each fund manager module has to implement this trait
#[async_trait]
pub trait FundManager: Send {
    async fn new() -> Result<Self, Error>
    where
        Self: Sized;
    fn etfs_under_management(&self) -> Vec<ETFListItem>;
    async fn etf_details(&mut self, ticker: &String) -> Result<ETF, Error>;
}

/// Common error type
#[derive(Debug)]
pub enum Error {
    Generic(String),
    NotFound,
}

impl<T: Display> From<T> for Error {
    fn from(error: T) -> Self {
        Error::Generic(error.to_string())
    }
}
