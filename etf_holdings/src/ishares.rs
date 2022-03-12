use async_trait::async_trait;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;

use crate::common::{ETFListItem, Error, FundManager, Holding, ETF};
use crate::deserialize_weird_floats;
use crate::exchange;

#[derive(Debug)]
pub struct IshareETFListItem {
    ticker: String,
    name: String,
    url: String,
}

#[derive(Debug)]
pub struct Ishare {
    fetched_etfs: HashMap<String, ETF>,
    etf_list: HashMap<String, IshareETFListItem>,
}

#[async_trait]
impl FundManager for Ishare {
    async fn new() -> Result<Self, Error> {
        let etf_list = {
            match fetch_etf_list().await {
                Ok(x) => x,
                Err(err) => {
                    return Err(Error::from(format!(
                        "Error Ishare::fetch_etf_list(): {:?}",
                        err
                    )))
                }
            }
        };
        Ok(Ishare {
            fetched_etfs: HashMap::new(),
            etf_list: etf_list,
        })
    }

    fn etfs_under_management(&self) -> Vec<ETFListItem> {
        self.etf_list
            .values()
            .map(|s| ETFListItem {
                ticker: s.ticker.clone(),
                name: s.name.clone(),
            })
            .collect()
    }

    async fn etf_details(&mut self, ticker: &String) -> Result<ETF, Error> {
        if let Some(etf) = self.fetched_etfs.get(ticker) {
            return Ok(etf.clone());
        }

        let etf_item = self
            .etf_list
            .get(ticker)
            .ok_or(format!("{} not found in iShare fund manager.", ticker))?;
        match fetch_holdings(&etf_item).await {
            Ok(etf) => {
                self.fetched_etfs.insert(ticker.clone(), etf);
                Ok(self.fetched_etfs.get(ticker).unwrap().clone())
            }
            Err(x) => Err(Error::from(format!(
                "Error Ishare::etf_details({}): {:?}",
                ticker, x
            ))),
        }
    }
}

async fn fetch_etf_list() -> Result<HashMap<String, IshareETFListItem>, Error> {
    let url = "https://www.ishares.com/us/products/etf-investments";
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    // The table we're looking for is in a noscript block.
    // We need to pull out the text inside and parse as HTML
    let noscript_selector = Selector::parse("noscript").unwrap();
    let noscript_html = document
        .select(&noscript_selector)
        .next()
        .ok_or("Can't find a noscript block on the iShare ETFs page.")?
        .text()
        .next()
        .ok_or("Found a noscript block on the iShare ETFs page, but no text nodes inside.")?;
    let noscript_fragment = Html::parse_fragment(&noscript_html);

    // Find the table of ETFs inside the noscript block
    // The table looks like the following, so we'll call next() on the iterator twice for each ETF
    //
    // <tr>
    // <td class="links"><a href="/us/products/239423/ishares-10-year-credit-bond-etf">IGLB</a></td>
    // <td class="links"><a href="/us/products/239423/ishares-10-year-credit-bond-etf">iShares 10+ Year Investment Grade Corporate Bond ETF</a></td>
    // <td class="column-left-line">3.53</td>
    // ..
    let tag_selector = Selector::parse("table > tbody > tr > td.links a").unwrap();
    let mut table_iterator = noscript_fragment.select(&tag_selector);

    let mut etfs = HashMap::new();
    while let Some(ticker_elem) = table_iterator.next() {
        let name = table_iterator
            .next()
            .ok_or("The iShare ETFs table ran out of name rows before ticker rows... weird")?
            .inner_html();
        let url = ticker_elem
            .value()
            .attr("href")
            .ok_or("No href on iShare ETFs table cell.")?
            .to_string();
        let ticker = ticker_elem.inner_html();
        etfs.insert(
            String::from(&ticker),
            IshareETFListItem {
                ticker: ticker,
                name: name,
                url: url,
            },
        );
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

async fn fetch_holdings(etf_item: &IshareETFListItem) -> Result<ETF, Error> {
    let url = format!(
        "https://www.ishares.com{}/1467271812596.ajax?fileType=csv&dataType=fund",
        etf_item.url
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
        let info_table = splitted_csv
            .next()
            .ok_or("Can't find iShare info table. CSV format must have changed.")?;
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
        let holdings_table = splitted_csv
            .next()
            .ok_or("Can't find iShare holdings table. CSV format must have changed.")?;
        let mut reader = csv::ReaderBuilder::new().from_reader(holdings_table.as_bytes());
        for record in reader.deserialize() {
            let row: IshareHolding = record?;
            let ticker = {
                if let Some(ticker_with_suffix) =
                    exchange::ticker_with_exchange_suffix(&row.ticker, &row.exchange)
                {
                    ticker_with_suffix
                } else {
                    println!("Couldn't find a suffix for exchange {}.", &row.exchange);
                    row.ticker
                }
            };
            holdings.push(Holding {
                ticker: ticker,
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
        ticker: etf_item.ticker.clone(),
        name: etf_item.name.clone(),
        last_update: last_update
            .ok_or("No last update found in iShare info table. CSV format must have changed.")?,
        outstanding_shares: outstanding_shares.ok_or(
            "No outstanding shares found in iShare info table. CSV format must have changed.",
        )?,
        holdings: holdings,
    })
}
