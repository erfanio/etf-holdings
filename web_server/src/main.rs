#[macro_use]
extern crate rocket;

use etf_holdings::{ETFListItem};
use rocket::serde::json::Json;
use rocket::State;

mod data;
mod util;
mod yahoo;
use data::{Cache, ETFDetails};
use util::Result;

#[get("/etf/list")]
async fn list(cache: &State<Cache>) -> Json<Vec<ETFListItem>> {
    Json(cache.etf_list().await)
}

#[get("/etf/<ticker>")]
async fn details(cache: &State<Cache>, ticker: String) -> Result<Json<ETFDetails>> {
    Ok(Json(cache.details(&ticker).await?))
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(Cache::new().await)
        .mount("/api", routes![list, details])
}
