use etf_holdings::{AvailableETFs, Error as ETFErr};
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::util::{Error, Result};
use crate::yahoo::{fetch_historical_prices, HistoricalPrices};

#[derive(Serialize, Debug, Clone)]
pub struct ETFDetails {
    pub equity: Vec<HoldingEquity>,
    pub other: HashMap<String, f64>,
    pub prices: Vec<HistoricalPrices>,
}

#[derive(Serialize, Debug, Clone)]
pub struct HoldingEquity {
    pub ticker: String,
    pub yahoo_symbol: Option<String>,
    pub name: String,
    pub weight: f64,
    pub location: String,
    pub exchange: String,
    pub prices: Vec<HistoricalPrices>,
}

pub struct Cache {
    etfs: AvailableETFs,
    etfs_cache: RwLock<HashMap<String, ETFDetails>>,
    prices_cache: RwLock<HashMap<String, Vec<HistoricalPrices>>>,
}

impl Cache {
    pub async fn new() -> Self {
        Cache {
            etfs: AvailableETFs::new().await,
            etfs_cache: RwLock::new(HashMap::new()),
            prices_cache: RwLock::new(HashMap::new()),
        }
    }
    pub async fn etf_list(&self) -> Vec<String> {
        self.etfs.etf_list().await
    }

    async fn prices(&self, ticker: &String) -> Result<Vec<HistoricalPrices>> {
        {
            let prices_cache = self.prices_cache.read().await;
            if let Some(cached) = prices_cache.get(ticker) {
                println!("Cached prices for {}!", ticker);
                return Ok(cached.clone());
            }
        }
        println!("Prices for {} not cached :(", ticker);

        let prices = {
            match fetch_historical_prices(&ticker).await {
                Ok(x) => x,
                Err(err) => {
                    return Err(Error::Generic(format!(
                        "Error Yahoo::fetch_historical_prices({}): {:?}",
                        ticker, err
                    )))
                }
            }
        };
        {
            let mut prices_cache = self.prices_cache.write().await;
            prices_cache.insert(ticker.clone(), prices.clone());
        }

        Ok(prices)
    }

    pub async fn details(&self, ticker: &String) -> Result<ETFDetails> {
        {
            let etfs_cache = self.etfs_cache.read().await;
            if let Some(cached) = etfs_cache.get(ticker) {
                println!("Cached details for {}!", ticker);
                return Ok(cached.clone());
            }
        }
        println!("Details for {} not cached :(", ticker);

        let etf = {
            match self.etfs.etf_details(ticker).await {
                Ok(x) => x,
                Err(ETFErr::NotFound) => {
                    return Err(Error::NotFound(format!(
                        "Can't find {} in AvailableETFs.",
                        ticker
                    )))
                }
                Err(ETFErr::Generic(msg)) => {
                    return Err(Error::Generic(format!(
                        "Error AvailableETFs::etf_details({}): {:?}",
                        ticker, msg
                    )))
                }
            }
        };

        let mut equity = Vec::new();
        let mut other = HashMap::new();
        for holding in etf.holdings {
            if holding.asset_class == "Equity" {
                let prices = {
                    // Test out fetching prices
                    if holding.ticker == "PLUG" {
                        self.prices(&holding.ticker).await?
                    } else {
                        vec![]
                    }
                };
                equity.push(HoldingEquity {
                    ticker: holding.ticker,
                    yahoo_symbol: holding.yahoo_symbol,
                    name: holding.name,
                    weight: holding.weight,
                    location: holding.location,
                    exchange: holding.exchange,
                    prices,
                });
            } else {
                let weight = other.entry(holding.asset_class.clone()).or_insert(0.0);
                *weight += holding.weight;
            }
        }

        let etf = ETFDetails {
            equity,
            other,
            prices: vec![],
        };
        {
            let mut etfs_cache = self.etfs_cache.write().await;
            etfs_cache.insert(ticker.clone(), etf.clone());
        }

        Ok(etf)
    }
}
