//! Provide a caching layer between the public web API, and the library

use etf_holdings::{ETFHoldings, ETFListItem, Error as ETFErr};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::types::{
    ChartHoldingDetails, ChartResponse, DetailsEquityHolding, DetailsResponse, GoodError,
    HistoricalPrices, GoodResult,
};
use crate::util::create_price_chart;
use crate::yahoo::fetch_historical_prices;

/// Generates responses for the web_server and caches ETF details and price history.
///
/// TODO: Split the business logic in Cache into its own module.
pub struct Cache {
    etf_holdings: ETFHoldings,
    etfs_cache: RwLock<HashMap<String, DetailsResponse>>,
    prices_cache: RwLock<HashMap<String, Vec<HistoricalPrices>>>,
}

impl Cache {
    /// Create a new instance of `Cache`
    pub async fn new() -> Self {
        Cache {
            etf_holdings: ETFHoldings::new().await,
            etfs_cache: RwLock::new(HashMap::new()),
            prices_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Return a list of ETFs supported by the ETF Holdings library.
    pub async fn etf_list(&self) -> Vec<ETFListItem> {
        self.etf_holdings.etf_list().await
    }

    /// Fetch the price history for a stock.
    async fn prices(&self, ticker: &String) -> GoodResult<Vec<HistoricalPrices>> {
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
                    return Err(GoodError::Generic(format!(
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

    /// Fetch ETF/holdings details and prices to create a DetailsResponse.
    ///
    /// All fetched prices and the final response is cached.
    pub async fn details(&self, ticker: &String) -> GoodResult<DetailsResponse> {
        {
            let etfs_cache = self.etfs_cache.read().await;
            if let Some(cached) = etfs_cache.get(ticker) {
                println!("Cached details for {}!", ticker);
                return Ok(cached.clone());
            }
        }
        println!("Details for {} not cached :(", ticker);

        let etf = {
            match self.etf_holdings.etf_details(ticker).await {
                Ok(x) => x,
                Err(ETFErr::NotFound) => {
                    return Err(GoodError::NotFound(format!(
                        "Can't find {} in ETFHoldings.",
                        ticker
                    )))
                }
                Err(ETFErr::Generic(msg)) => {
                    return Err(GoodError::Generic(format!(
                        "Error ETFHoldings::etf_details({}): {:?}",
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

                equity_holdings.push(DetailsEquityHolding {
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
        let etf = DetailsResponse {
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

    /// Use Cache::details() to fetch ETF details and create a ChartResponse from it.
    pub async fn chart(&self, ticker: &String) -> GoodResult<ChartResponse> {
        println!("Chart {}: Loading details.", ticker);
        let details = self.details(ticker).await?;
        println!("Chart {}: Details loaded.", ticker);

        let mut holding_details: HashMap<String, ChartHoldingDetails> = HashMap::new();
        for holding in &details.equity_holdings {
            holding_details.insert(
                holding.ticker.clone(),
                ChartHoldingDetails {
                    ticker: holding.ticker.clone(),
                    name: holding.name.clone(),
                },
            );
        }

        println!("Chart {}: Merging prices.", ticker);
        let chart = create_price_chart(&details)?;

        let result = ChartResponse {
            etf_ticker: details.ticker,
            etf_name: details.name,
            holding_details,
            chart,
        };
        println!("Chart {}: Final results\n{:#?}", ticker, result);
        Ok(result)
    }
}
