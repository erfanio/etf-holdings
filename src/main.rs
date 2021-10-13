#[macro_use] extern crate rocket;

mod data;

use rocket::State;
use rocket::serde::json::Json;
use serde::{Serialize};

#[derive(Serialize)]
struct JsonListItem {
    symbol: String,
    name: String,
}

#[get("/list")]
async fn symbols(cache: &State<data::Cache>) -> Option<Json<Vec<JsonListItem>>> {
    let symbols_codes = cache.get_symbols().await?
        .iter()
        .filter(|symb| symb.asset_type == "etf")
        .map(|symb| JsonListItem {
            symbol: symb.symbol.clone(),
            name: symb.name.clone(),
        })
        .collect();
    Some(Json(symbols_codes))
}

#[derive(Serialize)]
struct JsonHoldingItem {
    symbol: String,
    name: String,
}

#[get("/holdings/<ticker>")]
async fn holdings(cache: &State<data::Cache>, ticker: &str) -> Option<Json<Vec<String>>> {
    let assets = cache.get_holdings(ticker).await?
        .iter()
        .map(|hld| format!("{} {}%", hld.asset, hld.weight_percentage))
        .collect();
    Some(Json(assets))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(data::Cache::empty())
        .mount("/api", routes![symbols, holdings])
}