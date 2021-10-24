#[macro_use]
extern crate rocket;

use etf_holdings::AvailableETFs;
use rocket::serde::json::Json;
use rocket::State;

mod util;
use util::ok_or_log;

#[get("/etf/list")]
async fn list(etfs: &State<AvailableETFs>) -> Json<Vec<String>> {
    let list = etfs.etf_list().await;
    Json(list)
}

#[get("/etf/<ticker>")]
async fn details(etfs: &State<AvailableETFs>, ticker: String) -> Option<Json<Vec<String>>> {
    let holdings = ok_or_log(etfs.etf_details(ticker).await)?
        .holdings
        .iter()
        .map(|hld| format!("{} {}%", hld.ticker, hld.weight))
        .collect();
    Some(Json(holdings))
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(AvailableETFs::new().await)
        .mount("/api", routes![list, details])
}
