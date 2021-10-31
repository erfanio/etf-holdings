use serde::{Deserialize, Serialize};

use crate::util::AnyError;

// The yahoo response is annoyingly nested so there's gonna be quite a few structs
#[derive(Deserialize, Debug)]
pub struct ChartResponse {
    pub chart: Chart,
}

#[derive(Deserialize, Debug)]
pub struct Chart {
    pub result: Vec<ChartResult>,
}

#[derive(Deserialize, Debug)]
pub struct ChartResult {
    pub timestamp: Vec<i64>,
    pub indicators: Indicators,
}

#[derive(Deserialize, Debug)]
pub struct Indicators {
    pub quote: Vec<Quote>,
    pub adjclose: Vec<Adjclose>,
}

#[derive(Deserialize, Debug)]
pub struct Quote {
    pub volume: Vec<i64>,
    pub open: Vec<f64>,
    pub low: Vec<f64>,
    pub high: Vec<f64>,
    pub close: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Adjclose {
    pub adjclose: Vec<f64>,
}
// </yahoo response object>

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoricalPrices {
    timestamp: i64,
    volume: i64,
    open: f64,
    low: f64,
    high: f64,
    close: f64,
    adjclose: f64,
}

pub async fn fetch_historical_prices(ticker: &String) -> Result<Vec<HistoricalPrices>, AnyError> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1mo",
        ticker
    );
    let resp: ChartResponse = reqwest::get(url).await?.json().await?;

    let result = resp
        .chart
        .result
        .get(0)
        .ok_or("Yahoo response had no results.")?;
    let quote = result
        .indicators
        .quote
        .get(0)
        .ok_or("Yahoo response had no quotes.")?;
    let adjclose = result
        .indicators
        .adjclose
        .get(0)
        .ok_or("Yahoo response had no adjclose.")?;
    let iter = result
        .timestamp
        .iter()
        .zip(quote.volume.iter())
        .zip(quote.open.iter())
        .zip(quote.low.iter())
        .zip(quote.high.iter())
        .zip(quote.close.iter())
        .zip(adjclose.adjclose.iter());

    let mut history = Vec::new();
    for ((((((timestamp, volume), open), low), high), close), adjclose) in iter {
        history.push(HistoricalPrices {
            timestamp: timestamp.clone(),
            volume: volume.clone(),
            open: open.clone(),
            low: low.clone(),
            high: high.clone(),
            close: close.clone(),
            adjclose: adjclose.clone(),
        })
    }

    // Sanity check
    if history.len() != result.timestamp.len() {
        return Err(AnyError::from(format!(
            "history.len() {} != {} timestamp.len()",
            history.len(),
            result.timestamp.len()
        )));
    }

    Ok(history)
}
