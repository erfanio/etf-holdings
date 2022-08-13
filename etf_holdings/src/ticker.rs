use lazy_static::lazy_static;
use std::collections::HashMap;

// Static mapping of exchange names to the suffix used in yahoo symbols
// https://help.yahoo.com/kb/SLN2310.html
lazy_static! {
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

    static ref TICKER_OVERRIDE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("451.HK", "0451.HK");
        m.insert("968.HK", "0968.HK");
        m
    };
}

pub fn ticker_with_exchange_suffix(ticker: &String, exchange_name: &String) -> Option<String> {
    match (*YAHOO_EXCHANGE_SUFFIX).get(exchange_name.as_str()) {
        Some(suffix) => {
            let full_ticker = format!("{}{}", ticker, suffix);

            match (*TICKER_OVERRIDE).get(full_ticker.as_str()) {
                Some(new_ticker) => Some(new_ticker.to_string()),
                None => Some(full_ticker),
            }
        }
        None => None,
    }
}
