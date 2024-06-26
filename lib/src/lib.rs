//! A library for fetching ETF details and holdings.
//!
//! ETFHoldings provides an interface to discover supported ETFs and fetch their details.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

mod deserialize_formatted_floats;
mod ishares;
mod ticker;
mod types;
use ishares::Ishare;
pub use types::{ETFListItem, Error, FundManager, ETF};

/// An instance of `ETFHoldings` can list supported ETFs and fetch ETF details.
pub struct ETFHoldings {
    etf_to_manager: RwLock<HashMap<String, Arc<Mutex<dyn FundManager>>>>,
    etf_list: RwLock<Vec<ETFListItem>>,
}

impl ETFHoldings {
    /// Creates an instance of ETFHoldings. This includes network calls to find an up to date list
    /// of ETFs.
    pub async fn new() -> ETFHoldings {
        let mut etf_to_manager = HashMap::<String, Arc<Mutex<dyn FundManager>>>::new();
        let mut etf_list = Vec::<ETFListItem>::new();

        let manager_constructors = [Ishare::new];
        for manager_constructor in manager_constructors.iter() {
            match manager_constructor().await {
                Ok(manager) => {
                    let etfs = manager.etfs_under_management();
                    let manager_ref = Arc::new(Mutex::new(manager));
                    for etf in etfs {
                        etf_to_manager.insert(etf.ticker.clone(), manager_ref.clone());
                        etf_list.push(etf.clone());
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }

        ETFHoldings {
            etf_to_manager: RwLock::new(etf_to_manager),
            etf_list: RwLock::new(etf_list),
        }
    }

    /// Returns a list of supported ETFs.
    pub async fn etf_list(&self) -> Vec<ETFListItem> {
        self.etf_list
            .read()
            .await
            .iter()
            .map(|s| s.clone())
            .collect()
    }

    /// Fetch ETF details and holdings for a supported ETF.
    pub async fn etf_details(&self, ticker: &String) -> Result<ETF, Error> {
        let manager_map = self.etf_to_manager.read().await;
        let mut manager = manager_map.get(ticker).ok_or(Error::NotFound)?.lock().await;
        manager.etf_details(ticker).await
    }
}
