// This file contain different common types, structs, errors...
use async_trait::async_trait;
use std::fmt::Display;

// Shared models used by the fund specific code
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ETF {
    pub ticker: String,
    pub last_update: String,
    pub outstanding_shares: f64,
    pub holdings: Vec<Holding>,
}

// Trait each fund module has to implement
#[async_trait]
pub trait FundManager: Send {
    async fn new() -> Result<Self, Error>
    where
        Self: Sized;
    fn etfs_under_management(&self) -> Vec<String>;
    async fn etf_details(&mut self, ticker: &String) -> Result<ETF, Error>;
}

// Common error type
#[derive(Debug)]
pub struct Error {
    pub err: String,
}

impl<T: Display> From<T> for Error {
    fn from(error: T) -> Self {
        Error {
            err: error.to_string(),
        }
    }
}

impl Error {
    pub fn new<T: Display>(msg: T) -> Self {
        Error {
            err: msg.to_string(),
        }
    }
}
