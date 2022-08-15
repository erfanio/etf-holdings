use chrono::{NaiveDateTime, Timelike};
use serde::Deserialize;

use crate::types::{AnyError, HistoricalPrices};

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
    pub meta: Meta,
    pub timestamp: Vec<i64>,
    pub indicators: Indicators,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub gmtoffset: i64,
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
        // remove the timezone offset and make all timestamp in GMT then use the
        // midnight timestamp because we don't really care about when the markets close
        let ts_normalised = NaiveDateTime::from_timestamp(timestamp - result.meta.gmtoffset, 0);
        let midnight_ts =
            ts_normalised.timestamp() - i64::from(ts_normalised.time().num_seconds_from_midnight());

        history.push(HistoricalPrices {
            timestamp: midnight_ts,
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
