use serde::{Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

const FMP_BASE_URL:&'static str = "https://financialmodelingprep.com/api/v3";
const FMP_KEY:&'static str = "<redacted>";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldingItem {
    pub asset: String,
    pub name: String,
    pub shares_number: i64,
    pub weight_percentage: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolItem {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    #[serde(rename = "type")]
    pub asset_type: String,
}

async fn fetch_symbols() -> reqwest::Result<Vec<SymbolItem>> {
    reqwest::get(format!("{}/stock/list?apikey={}", FMP_BASE_URL, FMP_KEY))
        .await?
        .json::<Vec<SymbolItem>>()
        .await
}

async fn fetch_holdings(ticker: &str) -> reqwest::Result<Vec<HoldingItem>> {
    reqwest::get(format!("{}/etf-holder/{}?apikey={}", FMP_BASE_URL, ticker, FMP_KEY))
        .await?
        .json::<Vec<HoldingItem>>()
        .await
}

pub struct Cache {
    // A list of symbols. In a mutex so we're able to update safely across different threads.
    pub symbols: Mutex<Vec<SymbolItem>>,
    // Map of holdings in a mutex. Each holding cache item is in an Arc so we can return the
    // individual items without a mutex guard
    pub holdings: Mutex<HashMap<String, Arc<HoldingsCache>>>,
}

type SymbolsCache = Vec<SymbolItem>;
type HoldingsCache = Vec<HoldingItem>;

impl Cache {
    pub fn empty() -> Cache {
        Cache {
            symbols: Mutex::new(vec![]),
            holdings: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_symbols(&self) -> Option<MutexGuard<'_, SymbolsCache>> {
        // Lock for a short period only to inspect if there is a cache
        let in_cache = {
            match self.symbols.lock() {
                Ok(symb) => (*symb).len() > 0,
                Err(_) => false,
            }
        };
        // Fetch symbols only if no cache was found
        let fetched_symbols: Option<SymbolsCache> = {
            if !in_cache {
                println!("No cache symbols");
                match fetch_symbols().await {
                    Ok(results) => Some(results),
                    Err(e) => {
                        eprintln!("Failed to fetch symbols: {}", e);
                        None
                    }
                }
            } else {
                println!("Cache symbols");
                None
            }
        };
        // Update the cache with the symbols we just fetch (if we did)
        match self.symbols.lock() {
            Ok(mut symb) => {
                if let Some(new_symb) = fetched_symbols {
                    *symb = new_symb;
                }
                Some(symb)
            },
            Err(_) => None,
        }
    }

    pub async fn get_holdings(&self, ticker: &str) -> Option<Arc<HoldingsCache>> {
        // Lock for a short period only to inspect if there is a cache
        let in_cache = {
            match self.holdings.lock() {
                Ok(hld_map) => (*hld_map).contains_key(ticker),
                Err(_) => false,
            }
        };
        // Fetch holdings only if no cache was found
        let fetched_holdings: Option<HoldingsCache> = {
            if !in_cache {
                println!("No cache holdings");
                match fetch_holdings(ticker).await {
                    Ok(results) => Some(results),
                    Err(e) => {
                        eprintln!("Failed to fetch holdings: {}", e);
                        None
                    }
                }
            } else {
                println!("Cache holdings");
                None
            }
        };
        // Update the cache with the holdings we just fetch (if we did)
        match self.holdings.lock() {
            Ok(mut hld_map) => {
                if let Some(new_hld) = fetched_holdings {
                    (*hld_map).insert(ticker.to_string(), Arc::new(new_hld));
                }
                // Return a cloned Arc pointing to the holding data
                (*hld_map).get(ticker).map(|h| h.clone())
            },
            Err(_) => None,
        }
    }
}