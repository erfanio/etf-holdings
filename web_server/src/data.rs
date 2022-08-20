use etf_holdings::{AvailableETFs, ETFListItem, Error as ETFErr};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::types::{
    ETFChart, ETFChartHoldingDetails, ETFDetails, EquityDetails, Error, HistoricalPrices, Result,
};
use crate::util::create_price_chart;
use crate::yahoo::fetch_historical_prices;

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
                // Fetch price history for equity holdings
                // let prices = {
                //     if holding.ticker == "PLUG" || holding.ticker == "ORSTED.CO" {
                //         self.prices(&holding.ticker).await.ok()
                //     } else {
                //         None
                //     }
                // };
                let prices = self.prices(&holding.ticker).await.ok();

                equity_holdings.push(EquityDetails {
                    ticker: holding.ticker,
                    name: holding.name,
                    weight: holding.weight,
                    location: holding.location,
                    exchange: holding.exchange,
                    prices,
                });
            } else {
                // Add up other types of holdings
                let weight = other_holdings
                    .entry(holding.asset_class.clone())
                    .or_insert(0.0);
                *weight += holding.weight;
            }
        }

        let prices = self.prices(&ticker).await.ok();
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
        println!("Chart {}: Loading details.", ticker);
        let details = self.details(ticker).await?;
        println!("Chart {}: Details loaded.", ticker);

        let mut holding_details: HashMap<String, ETFChartHoldingDetails> = HashMap::new();
        for holding in &details.equity_holdings {
            holding_details.insert(
                holding.ticker.clone(),
                ETFChartHoldingDetails {
                    ticker: holding.ticker.clone(),
                    name: holding.name.clone(),
                },
            );
        }

        println!("Chart {}: Merging prices.", ticker);
        let chart = create_price_chart(&details)?;

        let result = ETFChart {
            etf_ticker: details.ticker,
            etf_name: details.name,
            holding_details,
            chart,
        };
        println!("Chart {}: Final results\n{:#?}", ticker, result);
        Ok(result)
    }
}
