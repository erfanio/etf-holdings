use etf_holdings::{AvailableETFs, ETFListItem, Error as ETFErr};
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::util::{merge_timestamps, Error, Result};
use crate::yahoo::{fetch_historical_prices, HistoricalPrices};

// ETFChart is the structure that contains all the information used to draw the historical holding breakdown chart
#[derive(Serialize, Debug, Clone)]
pub struct ETFChart {
    pub etf_ticker: String,
    pub etf_name: String,
    pub holding_tickers: Vec<String>,
    pub holding_names: Vec<String>,
    pub holding_weights: Vec<f64>,
    pub timestamps: Vec<i64>,
    pub etf_prices: Vec<Option<f64>>,
    pub holding_prices: Vec<Vec<Option<f64>>>,
}

// ETFDetails (and EquityDetails) format all the available information into an easy to use
// structure
#[derive(Serialize, Debug, Clone)]
pub struct ETFDetails {
    pub ticker: String,
    pub name: String,
    pub equity_holdings: Vec<EquityDetails>,
    pub other_holdings: HashMap<String, f64>,
    pub prices: Vec<HistoricalPrices>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EquityDetails {
    pub ticker: String,
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
    pub async fn etf_list(&self) -> Vec<ETFListItem> {
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

        let mut equity_holdings = Vec::new();
        let mut other_holdings = HashMap::new();
        for holding in etf.holdings {
            if holding.asset_class == "Equity" {
                let prices = {
                    // Test out fetching prices
                    if holding.ticker == "PLUG" || holding.ticker == "ORSTED.CO" {
                        self.prices(&holding.ticker).await?
                    } else {
                        vec![]
                    }
                };

                // let prices = self.prices(&holding.ticker).await?;
                equity_holdings.push(EquityDetails {
                    ticker: holding.ticker,
                    name: holding.name,
                    weight: holding.weight,
                    location: holding.location,
                    exchange: holding.exchange,
                    prices,
                });
            } else {
                let weight = other_holdings
                    .entry(holding.asset_class.clone())
                    .or_insert(0.0);
                *weight += holding.weight;
            }
        }

        let prices = self.prices(&ticker).await?;
        let etf = ETFDetails {
            ticker: etf.ticker,
            name: etf.name,
            equity_holdings,
            other_holdings,
            prices,
        };
        {
            let mut etfs_cache = self.etfs_cache.write().await;
            etfs_cache.insert(ticker.clone(), etf.clone());
        }

        Ok(etf)
    }

    pub async fn chart(&self, ticker: &String) -> Result<ETFChart> {
        let details = self.details(ticker).await?;

        let mut all_prices = vec![];
        let mut holding_tickers = vec![];
        let mut holding_names = vec![];
        let mut holding_weights = vec![];
        for holding in details.equity_holdings {
            holding_tickers.push(holding.ticker);
            holding_names.push(holding.name);
            holding_weights.push(holding.weight);
            all_prices.push(holding.prices);
        }

        // Merge all prices to use the same timestamp axis. Days without price data for all
        // holdings and the ETF become obvious after doing this.
        // Create a Vec like [holding1, holding2, ..., etf]
        // Having the etf at the end is important so its timestamp is merged with holdings too
        all_prices.push(details.prices);
        let (timestamps, merged_prices) = merge_timestamps(all_prices);
        let merged_close_prices: Vec<Vec<Option<f64>>> = merged_prices
            .iter()
            .map(|x| x.iter().map(|y| Some(y.clone()?.close)).collect())
            .collect();

        let holding_prices = merged_close_prices
            .get(0..holding_tickers.len())
            .unwrap()
            .to_vec();
        let etf_prices = merged_close_prices.last().unwrap().to_vec();

        Ok(ETFChart {
            etf_ticker: details.ticker,
            etf_name: details.name,
            holding_tickers,
            holding_names,
            holding_weights,
            timestamps,
            etf_prices,
            holding_prices,
        })
    }
}
