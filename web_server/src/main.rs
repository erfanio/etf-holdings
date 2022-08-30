#[macro_use]
extern crate rocket;

use etf_holdings::ETFListItem;
use rocket::serde::json::Json;
use rocket::State;

mod cache;
mod types;
mod util;
mod yahoo;
use cache::Cache;
use types::{ChartResponse, DetailsResponse, GoodResult};

/// Handler for the list endpoint.
#[get("/etf/list")]
async fn list(cache: &State<Cache>) -> Json<Vec<ETFListItem>> {
    Json(cache.etf_list().await)
}

/// Handler for the details endpoint.
#[get("/etf/<ticker>")]
async fn details(cache: &State<Cache>, ticker: String) -> GoodResult<Json<DetailsResponse>> {
    Ok(Json(cache.details(&ticker).await?))
}

/// Handler for the chart endpoint.
#[get("/etf_chart/<ticker>")]
async fn chart(cache: &State<Cache>, ticker: String) -> GoodResult<Json<ChartResponse>> {
    Ok(Json(cache.chart(&ticker).await?))
}

/// The entry point of the binary.
#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(Cache::new().await)
        .mount("/api", routes![list, chart, details])
}
