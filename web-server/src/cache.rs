//! Provide a caching layer to save network requests.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::types::{DetailsResponse, GoodError, GoodResult, HistoricalPrices};
use crate::yahoo::fetch_historical_prices;

/// Cache for expensive to query objects.
pub struct Cache {
    details_cache: RwLock<HashMap<String, DetailsResponse>>,
    prices_cache: RwLock<HashMap<String, Vec<HistoricalPrices>>>,
}

impl Cache {
    /// Create a new instance of `Cache`.
    pub async fn new() -> Self {
        Cache {
            details_cache: RwLock::new(HashMap::new()),
            prices_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Fetch the price history for a stock.
    pub async fn prices(&self, ticker: &String) -> GoodResult<Vec<HistoricalPrices>> {
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

    /// Get ETF details from cache (if available).
    pub async fn get_details(&self, ticker: &String) -> Option<DetailsResponse> {
        let details_cache = self.details_cache.read().await;
        if let Some(cached) = details_cache.get(ticker) {
            println!("Cached details for {}!", ticker);
            return Some(cached.clone());
        }
        println!("Details for {} not cached :(", ticker);
        None
    }

    /// Set ETF details.
    pub async fn insert_details(&self, ticker: &String, details: &DetailsResponse) {
        let mut details_cache = self.details_cache.write().await;
        details_cache.insert(ticker.clone(), details.clone());
    }
}
