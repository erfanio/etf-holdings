use std::fmt::{Debug, Display};
use std::iter::Map;
use std::iter::Peekable;
use std::slice::Iter;

use crate::yahoo::HistoricalPrices;

// Generic error for other stuff. Implementing From on Error would cause everything to have generic messages and kind
// so this is here to clearly distinguish between generic errors and errors with good messages
#[derive(Debug)]
pub struct AnyError(String);

impl<T: Display> From<T> for AnyError {
    fn from(error: T) -> Self {
        AnyError(error.to_string())
    }
}

// Error type that emits good HTTP status
#[derive(Debug)]
pub enum Error {
    Generic(String),
    NotFound(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        warn_!("Error: {:?}", self);
        match self {
            Error::Generic(_) => Err(rocket::http::Status::InternalServerError),
            Error::NotFound(_) => Err(rocket::http::Status::NotFound),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// Merge many different equity prices to use the same timestamps
//
// Takes [ HistoricalPrices { timestamp: i64, ... }, ... ]
// Returns a list of timestamps and a list of HistoricalPrices for each timestamp
// prices.len() == merged_prices.len()
// timestamps.len() == merged_prices[0].len()
pub fn merge_timestamps(
    prices: Vec<Option<Vec<HistoricalPrices>>>,
) -> (Vec<i64>, Vec<Vec<Option<HistoricalPrices>>>) {
    // Convert all vectors into an iterator but keep Nones in the same position
    let mut iters: Vec<Option<Peekable<Iter<HistoricalPrices>>>> = prices
        .iter()
        .map(|o| match o {
            Some(x) => Some(x.iter().peekable()),
            None => None,
        })
        .collect();

    let mut merged_prices = vec![vec![]; prices.len()];
    let mut timestamps = vec![];

    // Each loop will find all items matching a particular timestamp and add them to merged_prices
    loop {
        // Find the lowest timestamp
        let maybe_current_timestamp = iters
            .iter_mut()
            // This bit clones the timestamp to avoid borrowing otherwise we won't be able borrow
            // again for the for loop
            .filter_map(|x| match x {
                Some(h) => Some(h.peek()?.timestamp.clone()),
                None => None,
            })
            .min();

        // We'll see a timestamp here until all iters have been exhausted
        if let Some(current_timestamp) = maybe_current_timestamp {
            // Look for matching timestamps in all the iters
            let mut new_prices = vec![];
            for maybe_iter in iters.iter_mut() {
                if let Some(iter) = maybe_iter {
                    // Add the iter value if the timestamp matches, None otherwise
                    if let Some(price) = iter.peek() {
                        if price.timestamp == current_timestamp {
                            new_prices.push(Some(Some(iter.next().unwrap().clone())));
                        } else {
                            new_prices.push(None);
                        }
                    } else {
                        new_prices.push(None);
                    }
                } else {
                    // This iter doesn't have any prices, so we want to add None to the final
                    // results but still keep this timestamp for everything that does have prices
                    new_prices.push(Some(None))
                }
            }

            // Only add timestamps that have prices for all entries
            if new_prices.iter().all(|x| x.is_some()) {
                timestamps.push(current_timestamp);
                for (i, new_price) in new_prices.into_iter().enumerate() {
                    merged_prices[i].push(new_price.unwrap());
                }
            }
        } else {
            break;
        }
    }
    (timestamps, merged_prices)
}
