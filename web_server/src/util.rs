use std::fmt::{Debug, Display};
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
pub fn merge_timestamps(
    prices: Vec<Vec<HistoricalPrices>>,
) -> (Vec<i64>, Vec<Vec<Option<HistoricalPrices>>>) {
    let mut iters: Vec<Peekable<Iter<HistoricalPrices>>> =
        prices.iter().map(|x| x.iter().peekable()).collect();

    let mut merged_prices = vec![vec![]; prices.len()];
    let mut timestamps = vec![];
    loop {
        // Find the the lowest timestamp
        let maybe_current_timestamp = iters
            .iter_mut()
            // This bit clones the timestamp to avoid borrowing
            // otherwise we won't be able borrow again for the for loop
            .filter_map(|x| Some(x.peek()?.timestamp.clone()))
            .min();

        // We'll see a timestamp here until all iters have been exhausted
        if let Some(current_timestamp) = maybe_current_timestamp {
            timestamps.push(current_timestamp);

            // Add prices that match the timestamp to the merged list
            for (i, iter) in iters.iter_mut().enumerate() {
                if let Some(price) = iter.peek() {
                    if price.timestamp == current_timestamp {
                        merged_prices[i].push(Some(iter.next().unwrap().clone()));
                    } else {
                        merged_prices[i].push(None);
                    }
                } else {
                    merged_prices[i].push(None);
                }
            }
        } else {
            break;
        }
    }
    (timestamps, merged_prices)
}
