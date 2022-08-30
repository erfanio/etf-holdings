//! Yahoo is our source of price history.

use chrono::{NaiveDateTime, Timelike};
use serde::Deserialize;

use crate::types::{to_good_error, GoodError, GoodResult, HistoricalPrices};

// The yahoo response is annoyingly nested so there's gonna be quite a few structs

/// Yahoo price history response type
#[derive(Deserialize, Debug)]
pub struct YahooResponse {
    pub chart: YahooChart,
}

#[derive(Deserialize, Debug)]
pub struct YahooChart {
    pub result: Vec<YahooResult>,
}

#[derive(Deserialize, Debug)]
pub struct YahooResult {
    pub meta: YahooMeta,
    pub timestamp: Vec<i64>,
    pub indicators: YahooIndicators,
}

#[derive(Deserialize, Debug)]
pub struct YahooMeta {
    pub gmtoffset: i64,
}

#[derive(Deserialize, Debug)]
pub struct YahooIndicators {
    pub quote: Vec<YahooQuote>,
    pub adjclose: Vec<YahooAdjclose>,
}

#[derive(Deserialize, Debug)]
pub struct YahooQuote {
    pub volume: Vec<i64>,
    pub open: Vec<f64>,
    pub low: Vec<f64>,
    pub high: Vec<f64>,
    pub close: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct YahooAdjclose {
    pub adjclose: Vec<f64>,
}
// </yahoo response object>

/// Fetch price history for a stock
pub async fn fetch_historical_prices(ticker: &String) -> GoodResult<Vec<HistoricalPrices>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=6mo",
        ticker
    );
    let resp: YahooResponse = reqwest::get(url)
        .await
        .map_err(to_good_error)?
        .json()
        .await
        .map_err(to_good_error)?;

    let result = resp
        .chart
        .result
        .get(0)
        .ok_or("Yahoo response had no results.")
        .map_err(to_good_error)?;
    let quote = result
        .indicators
        .quote
        .get(0)
        .ok_or("Yahoo response had no quotes.")
        .map_err(to_good_error)?;
    let adjclose = result
        .indicators
        .adjclose
        .get(0)
        .ok_or("Yahoo response had no adjclose.")
        .map_err(to_good_error)?;
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
        return Err(GoodError::Generic(format!(
            "history.len() {} != {} timestamp.len()",
            history.len(),
            result.timestamp.len()
        )));
    }

    Ok(history)
}
