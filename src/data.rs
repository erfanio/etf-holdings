use std::sync::{Mutex, MutexGuard};
use serde::{Serialize, Deserialize};

const FMP_BASE_URL:&'static str = "https://financialmodelingprep.com/api/v3";
const FMP_KEY:&'static str = "<redacted>";

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolItem {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub exchange: String,
    pub exchangeShortName: String,
}

impl SymbolItem {
    pub async fn fetch_symbols() -> reqwest::Result<Vec<SymbolItem>> {
        let symbols = reqwest::get(format!("{}/etf/list?apikey={}", FMP_BASE_URL, FMP_KEY))
            .await?
            .json::<Vec<SymbolItem>>()
            .await?;
        reqwest::Result::Ok(symbols)
    }
}

pub struct Cache {
    pub symbols: Mutex<Vec<SymbolItem>>,
}

impl Cache {
    pub fn empty() -> Cache {
        Cache {
            symbols: Mutex::<Vec<SymbolItem>>::new(vec![]),
        }
    }

    pub async fn get_symbols(&self) -> Option<MutexGuard<'_, Vec<SymbolItem>>> {
        // Lock for a short period only to inspect if there is a cache
        let in_cache = {
            match self.symbols.lock() {
                Ok(symb) => (*symb).len() > 0,
                Err(_) => false,
            }
        };
        // Fetch symbols only if no cache was found
        let fetched_symbols: Option<Vec<SymbolItem>> = {
            if !in_cache {
                println!("No cache");
                Some(SymbolItem::fetch_symbols().await.ok()?)
            } else {
                println!("Cache");
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
}