use async_trait::async_trait;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;

use crate::common::{Error, FundManager, Holding, ETF};
use crate::deserialize_weird_floats;

#[derive(Debug)]
pub struct Ishare {
    fetched_etfs: HashMap<String, ETF>,
    etf_urls: HashMap<String, String>,
}

#[async_trait]
impl FundManager for Ishare {
    async fn new() -> Result<Self, Error> {
        Ok(Ishare {
            fetched_etfs: HashMap::new(),
            etf_urls: fetch_etf_list().await?,
        })
    }

    fn etfs_under_management(&self) -> Vec<String> {
        self.etf_urls.keys().map(|s| s.clone()).collect()
    }

    async fn etf_details(&mut self, ticker: &String) -> Result<ETF, Error> {
        if let Some(etf) = self.fetched_etfs.get(ticker) {
            return Ok(etf.clone());
        }

        let url = self
            .etf_urls
            .get(ticker)
            .ok_or(format!("{} not found in iShare fund manager.", ticker))?;
        match fetch_holdings(ticker, &String::from(url)).await {
            Ok(etf) => {
                self.fetched_etfs.insert(ticker.clone(), etf);
                Ok(self.fetched_etfs.get(ticker).unwrap().clone())
            }
            Err(x) => Err(x),
        }
    }
}

async fn fetch_etf_list() -> Result<HashMap<String, String>, Error> {
    let url = "https://www.ishares.com/us/products/etf-investments";
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    // The table we're looking for is in a noscript block.
    // We need to pull out the text inside and parse as HTML
    let noscript_selector = Selector::parse("noscript").unwrap();
    let noscript_html = document
        .select(&noscript_selector)
        .next()
        .ok_or(Error::new(
            "Can't find a noscript block on the iShare ETFs page.",
        ))?
        .text()
        .next()
        .ok_or(Error::new(
            "Found a noscript block on the iShare ETFs page, but no text nodes inside.",
        ))?;
    let noscript_fragment = Html::parse_fragment(&noscript_html);

    // Find the table of ETFs inside the noscript block
    let a_tag_selector = Selector::parse("table > tbody > tr > td.links:first-child a").unwrap();
    let mut etfs = HashMap::new();
    for elem in noscript_fragment.select(&a_tag_selector) {
        let url = elem
            .value()
            .attr("href")
            .ok_or(Error::new("No href on iShare ETFs table cell."))?
            .to_string();
        let ticker = elem.inner_html();
        etfs.insert(ticker, url);
    }
    Ok(etfs)
}

#[derive(Debug, Deserialize)]
struct IshareHolding {
    #[serde(rename = "Ticker")]
    ticker: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Asset Class")]
    asset_class: String,
    #[serde(rename = "Market Value")]
    #[serde(with = "deserialize_weird_floats")]
    market_value: f64,
    #[serde(rename = "Weight (%)")]
    #[serde(with = "deserialize_weird_floats")]
    weight: f64,
    #[serde(rename = "Notional Value")]
    #[serde(with = "deserialize_weird_floats")]
    notional_value: f64,
    #[serde(rename = "Shares")]
    #[serde(with = "deserialize_weird_floats")]
    shares: f64,
    #[serde(rename = "Price")]
    #[serde(with = "deserialize_weird_floats")]
    price: f64,
    #[serde(rename = "Location")]
    location: String,
    #[serde(rename = "Exchange")]
    exchange: String,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "FX Rate")]
    #[serde(with = "deserialize_weird_floats")]
    fx_rate: f64,
    #[serde(rename = "Market Currency")]
    market_currency: String,
}

async fn fetch_holdings(ticker: &String, url_fragment: &String) -> Result<ETF, Error> {
    let url = format!(
        "https://www.ishares.com{}/1467271812596.ajax?fileType=csv&dataType=fund",
        url_fragment
    );
    let csv = reqwest::get(url)
        .await?
        .text()
        .await?
        .replace(|c: char| !c.is_ascii(), "");
    let mut splitted_csv = csv.split("\n\n");

    let mut outstanding_shares = None;
    let mut last_update = None;
    {
        let info_table = splitted_csv.next().ok_or(Error::new(
            "Can't find iShare info table. CSV format must have changed.",
        ))?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(info_table.as_bytes());
        for record in reader.records() {
            let row = record?;

            if row.len() > 1 {
                if row.get(0).unwrap() == "Fund Holdings as of" {
                    last_update = Some(row.get(1).unwrap().to_string());
                }
                if row.get(0).unwrap() == "Shares Outstanding" {
                    outstanding_shares = Some(deserialize_weird_floats::parse_weird_float(
                        row.get(1).unwrap(),
                    )?);
                }
            }
        }
    }

    let mut holdings = Vec::new();
    {
        let holdings_table = splitted_csv.next().ok_or(Error::new(
            "Can't find iShare holdings table. CSV format must have changed.",
        ))?;
        let mut reader = csv::ReaderBuilder::new().from_reader(holdings_table.as_bytes());
        for record in reader.deserialize() {
            let row: IshareHolding = record?;
            holdings.push(Holding {
                ticker: row.ticker,
                name: row.name,
                asset_class: row.asset_class,
                market_value: row.market_value,
                weight: row.weight,
                notional_value: row.notional_value,
                shares: row.shares,
                price: row.price,
                location: row.location,
                exchange: row.exchange,
                currency: row.currency,
                fx_rate: row.fx_rate,
                market_currency: row.market_currency,
            })
        }
    }
    Ok(ETF {
        ticker: ticker.clone(),
        last_update: last_update.ok_or(Error::new(
            "No last update found in iShare info table. CSV format must have changed.",
        ))?,
        outstanding_shares: outstanding_shares.ok_or(Error::new(
            "No outstanding shares found in iShare info table. CSV format must have changed.",
        ))?,
        holdings: holdings,
    })
}
