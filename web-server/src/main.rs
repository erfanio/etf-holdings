#[macro_use]
extern crate rocket;

use etf_holdings_lib::{ETFHoldings, ETFListItem};
use rocket::serde::json::Json;
use rocket::State;

mod cache;
mod chart;
mod details;
mod types;
mod yahoo;
use cache::Cache;
use chart::chart_response;
use details::details_response;
use types::{ChartResponse, DetailsResponse, GoodResult};

/// Handler for the list endpoint.
#[get("/etf/list")]
async fn list_handler(etf_holdings: &State<ETFHoldings>) -> Json<Vec<ETFListItem>> {
    Json(etf_holdings.etf_list().await)
}

/// Handler for the details endpoint.
#[get("/etf/<ticker>")]
async fn details_handler(
    cache: &State<Cache>,
    etf_holdings: &State<ETFHoldings>,
    ticker: String,
) -> GoodResult<Json<DetailsResponse>> {
    Ok(Json(
        details_response(&cache, &etf_holdings, &ticker).await?,
    ))
}

/// Handler for the chart endpoint.
#[get("/etf_chart/<ticker>")]
async fn chart_handler(
    cache: &State<Cache>,
    etf_holdings: &State<ETFHoldings>,
    ticker: String,
) -> GoodResult<Json<ChartResponse>> {
    Ok(Json(chart_response(&cache, &etf_holdings, &ticker).await?))
}

/// The entry point of the binary.
#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(Cache::new().await)
        .manage(ETFHoldings::new().await)
        .mount(
            "/api",
            routes![list_handler, chart_handler, details_handler],
        )
}
