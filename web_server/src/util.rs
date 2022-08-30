//! Util and helper functions

use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::types::{ChartPrice, DetailsResponse, GoodError, HistoricalPrices, GoodResult};

// Merge many different equity prices to use the same set of timestamps
pub fn merge_chart_prices(details: &DetailsResponse) -> GoodResult<Vec<ChartPrice>> {
    // Create a map of peekable iterator of prices and skip tickers with no price data
    let mut price_iters: HashMap<String, Peekable<Iter<HistoricalPrices>>> = HashMap::new();
    if let Some(etf_price) = &details.prices {
        price_iters.insert(details.ticker.clone(), etf_price.iter().peekable());
    } else {
        return Err(GoodError::Generic("ETF Price not available!".to_string()));
    }
    for holding in &details.equity_holdings {
        if let Some(holding_price) = &holding.prices {
            price_iters.insert(holding.ticker.clone(), holding_price.iter().peekable());
        }
    }

    let mut merged_prices: Vec<ChartPrice> = vec![];
    // Each loop will find all items matching a particular timestamp and add them to merged_prices
    loop {
        // Find the lowest timestamp
        let maybe_current_timestamp = price_iters
            .values_mut()
            // This bit clones the timestamp to avoid borrowing otherwise we won't be able borrow
            // again for the for loop
            .filter_map(|x| Some(x.peek()?.timestamp.clone()))
            .min();

        // We'll see a timestamp here until all iters have been exhausted
        if let Some(current_timestamp) = maybe_current_timestamp {
            // Look for matching timestamps in all the iters
            let mut new_prices: HashMap<String, f64> = HashMap::new();
            let mut missing_prices = false;
            for (ticker, iter) in price_iters.iter_mut() {
                // Add the iter value if the timestamp matches, flip missing_prices otherwise but
                // continue the loop so all matching timestamps are pulled out of iterators
                if let Some(price) = iter.peek() {
                    if price.timestamp == current_timestamp {
                        new_prices.insert(ticker.clone(), iter.next().unwrap().close);
                    } else {
                        missing_prices = true;
                    }
                } else {
                    missing_prices = true;
                }
            }

            // Ignore any timestamp with missing prices
            if !missing_prices {
                let etf_price = new_prices.remove(&details.ticker).unwrap();
                merged_prices.push(ChartPrice {
                    timestamp: current_timestamp,
                    etf_price,
                    holding_prices: new_prices,
                });
            }
        } else {
            break;
        }
    }

    Ok(merged_prices)
}

// Create a price history chart.
// * The Y-axis is the historical price as a percentage of first date's price (ETF's price is 100,
// each holding's price is equal to their weight)
// * The X-axis is the date
pub fn create_price_chart(details: &DetailsResponse) -> GoodResult<Vec<ChartPrice>> {
    let mut prices = merge_chart_prices(details)?;
    prices.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let first_price = prices.first().unwrap().clone();
    let last_price = prices.last().unwrap().clone();

    // The 100% is the first ETF price we see in the data but the weight of holdings is calculated
    // at today's price. i.e. if ETF is up 20%, ETF is at 120% and each holding is weight*1.2
    // We need to calculate a multiplier to go from the holding's price to our percentage axis.
    let weight_multiplier = last_price.etf_price / first_price.etf_price;
    let mut holding_multipliers: HashMap<String, f64> = HashMap::new();
    for holding in &details.equity_holdings {
        if let Some(holding_last_price) = last_price.holding_prices.get(&holding.ticker) {
            let normalized_weight = holding.weight * weight_multiplier;
            let multiplier = normalized_weight / holding_last_price;
            holding_multipliers.insert(holding.ticker.clone(), multiplier);
        }
    }

    // Normalise the prices to share the same Y-axis (as described in the previous block)
    for price in prices.iter_mut() {
        price.etf_price = price.etf_price / first_price.etf_price * 100.0;
        for (ticker, holding_price) in price.holding_prices.iter_mut() {
            *holding_price *= holding_multipliers.get(ticker).unwrap();
        }
    }

    Ok(prices)
}
