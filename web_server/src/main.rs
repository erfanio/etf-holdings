#[macro_use]
extern crate rocket;

use etf_holdings::ETFListItem;
use rocket::serde::json::Json;
use rocket::State;

mod data;
mod types;
mod util;
mod yahoo;
use data::Cache;
use types::{ETFChart, ETFDetails, Result};

#[get("/etf/list")]
async fn list(cache: &State<Cache>) -> Json<Vec<ETFListItem>> {
    Json(cache.etf_list().await)
}

#[get("/etf/<ticker>")]
async fn details(cache: &State<Cache>, ticker: String) -> Result<Json<ETFDetails>> {
    Ok(Json(cache.details(&ticker).await?))
}

#[get("/etf_chart/<ticker>")]
async fn chart(cache: &State<Cache>, ticker: String) -> Result<Json<ETFChart>> {
    Ok(Json(cache.chart(&ticker).await?))
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(Cache::new().await)
        .mount("/api", routes![list, chart, details])
}
