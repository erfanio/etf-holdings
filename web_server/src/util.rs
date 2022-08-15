use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::types::{ETFChartPrice, ETFDetails, HistoricalPrices};

// Merge many different equity prices to use the same set of timestamps
pub fn merge_chart_prices(details: &ETFDetails) -> Vec<ETFChartPrice> {
    // Create a map of peekable iterator of prices and skip tickers with no price data
    let mut price_iters: HashMap<String, Peekable<Iter<HistoricalPrices>>> = HashMap::new();
    if let Some(etf_price) = &details.prices {
        price_iters.insert(details.ticker.clone(), etf_price.iter().peekable());
    }
    for holding in &details.equity_holdings {
        if let Some(holding_price) = &holding.prices {
            price_iters.insert(holding.ticker.clone(), holding_price.iter().peekable());
        }
    }

    let mut merged_prices: Vec<ETFChartPrice> = vec![];
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
                let etf_price = new_prices.remove(&details.ticker);
                merged_prices.push(ETFChartPrice {
                    timestamp: current_timestamp,
                    etf_price,
                    holding_prices: new_prices,
                });
            }
        } else {
            break;
        }
    }

    merged_prices
}
