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

pub async fn get_symbols() -> reqwest::Result<Vec<SymbolItem>> {
    let symbols = reqwest::get(format!("{}/etf/list?apikey={}", FMP_BASE_URL, FMP_KEY))
        .await?
        .json::<Vec<SymbolItem>>()
        .await?;
    reqwest::Result::Ok(symbols)
}