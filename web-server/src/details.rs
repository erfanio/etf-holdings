//! Module used for constructing DetailsResponse.

use etf_holdings_lib::{ETFHoldings, Error as ETFErr};
use std::collections::HashMap;

use crate::cache::Cache;
use crate::types::{DetailsEquityHolding, DetailsResponse, GoodError, GoodResult};

/// Details response includes full ETF/holding details and price history.
///
/// The response is cached to save repeat network requests. All fetched prices are cached separate
/// to be used for other ETFs too.
pub async fn details_response(
    cache: &Cache,
    etf_holdings: &ETFHoldings,
    ticker: &String,
) -> GoodResult<DetailsResponse> {
    if let Some(response) = cache.get_details(ticker).await {
        return Ok(response);
    }

    let etf = {
        match etf_holdings.etf_details(ticker).await {
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
            let prices = cache.prices(&holding.ticker).await.ok();

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

    let prices = cache.prices(&ticker).await.ok();
    let response = DetailsResponse {
        ticker: etf.ticker,
        name: etf.name,
        equity_holdings,
        other_holdings,
        prices,
    };
    cache.insert_details(ticker, &response).await;
    Ok(response)
}
