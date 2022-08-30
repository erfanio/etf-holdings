//! Qualify tickers with exchange suffix

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Mapping of exchange names to the suffix used in yahoo symbols
    /// https://help.yahoo.com/kb/SLN2310.html
    static ref YAHOO_EXCHANGE_SUFFIX: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // United States
        m.insert("New York Stock Exchange Inc.", "");
        m.insert("NASDAQ", "");
        m.insert("Nyse Mkt Llc", "");
        m.insert("Cboe BZX formerly known as BATS", "");
        // Australia
        m.insert("Asx - All Markets", ".AX");
        // Denmark
        m.insert("Omx Nordic Exchange Copenhagen A/S", ".CO");
        // United Kingdown
        m.insert("London Stock Exchange", ".L");
        // Spain
        m.insert("Bolsa De Madrid", ".MC");
        // Portugal
        m.insert("Nyse Euronext - Euronext Lisbon", ".LS");
        // Hong Kong
        m.insert("Hong Kong Exchanges And Clearing Ltd", ".HK");
        // Austria
        m.insert("Wiener Boerse Ag", ".VI");
        // Germany
        m.insert("Xetra", ".DE");
        // Canada
        m.insert("Toronto Stock Exchange", ".TO");
        // South Korea
        m.insert("Korea Exchange (Stock Market)", ".KS");
        m.insert("Korea Exchange (Kosdaq)", ".KQ");
        // New Zealand
        m.insert("New Zealand Exchange Ltd", ".NZ");
        // Norway
        m.insert("Oslo Bors Asa", ".OL");
        // France
        m.insert("Nyse Euronext - Euronext Paris", ".PA");
        // Switzerland
        m.insert("SIX Swiss Exchange", ".SW");
        // Japan
        m.insert("Tokyo Stock Exchange", ".T");
        // Israel
        m.insert("Tel Aviv Stock Exchange", ".TA");
        // Italy
        m.insert("Borsa Italiana", ".MI");
        // Sweden
        m.insert("Nasdaq Omx Nordic", ".ST");
        // Netherlands
        m.insert("Euronext Amsterdam", ".AS");
        // Belgium
        m.insert("Nyse Euronext - Euronext Brussels", ".BR");
        // Finland
        m.insert("Nasdaq Omx Helsinki Ltd.", ".HE");
        // Singapore
        m.insert("Singapore Exchange", ".SI");
        // Ireland
        m.insert("Irish Stock Exchange - All Market", ".IR");
        m
    };

    /// Mapping of ticker aliases to the canonical ticker
    static ref TICKER_OVERRIDE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("451.HK", "0451.HK");
        m.insert("968.HK", "0968.HK");
        m
    };
}

/// Returns a fully qualified ticker that can be used for price information
///
/// We need a unique ticker for every stock across the world. Tickers aren't guaranteed to be
/// unique internationally but they are unique within their own stock exchange.
///
/// Since Yahoo is prevelant source of stock information, we'll copy their solution which adds a
/// stock exchange suffix to create a globally unique fully qualified ticker. Yahoo doesn't add a
/// suffix for US stocks (probably because they're guaranteed within the US) but all intenational
/// stocks have a suffix.
///
/// For example Yahoo `MEL.NZ` is a Kiwi company with ticker MEL listed on New Zealand Exchange
///
/// There are also quirky stocks that are often badly encoded or have aliases. This function will
/// correct those mistakes as well.
///
/// For example take `0968.HK`, XINYI SOLAR HOLDINGS LTD, the ticker, 0968, is interpreted as a
/// number in spreadsheets and "corrected" to 968. We have an override for `968.HK` -> `0968.HK`
///
/// Examples:
///
/// ```ignore
/// let mel = fully_qualified_ticker(
///     &"MEL".to_string(),
///     &"New Zealand Exchange Ltd".to_string(),
/// );
/// assert_eq!(mel, "MEL.NZ".to_string());
///
/// let xinyi = fully_qualified_ticker(
///     &"968".to_string(),
///     &"Hong Kong Exchanges And Clearing Ltd".to_string()
/// );
/// assert_eq!(xinyi, "0968.HK".to_string());
/// ```
pub fn fully_qualified_ticker(ticker: &String, exchange_name: &String) -> String {
    match (*YAHOO_EXCHANGE_SUFFIX).get(exchange_name.as_str()) {
        Some(suffix) => {
            let full_ticker = format!("{}{}", ticker, suffix);

            // Check if there is an override for this ticker
            match (*TICKER_OVERRIDE).get(full_ticker.as_str()) {
                Some(new_ticker) => new_ticker.to_string(),
                None => full_ticker,
            }
        }
        None => {
            println!("Couldn't find a suffix for exchange {}.", exchange_name);
            ticker.clone()
        }
    }
}
