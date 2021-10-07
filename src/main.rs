#[macro_use] extern crate rocket;

mod data;

use rocket::State;
use rocket::serde::json::Json;

#[get("/symbols")]
async fn symbols(cache: &State<data::Cache>) -> Option<Json<Vec<String>>> {
    let symbols_codes = cache.get_symbols().await?
        .iter()
        .map(|symb| String::clone(&symb.symbol))
        .collect();
    Some(Json(symbols_codes))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(data::Cache::empty())
        .mount("/api", routes![symbols])
}