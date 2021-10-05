#[macro_use] extern crate rocket;

mod data;

use std::sync::Mutex;
use rocket::State;
use rocket::serde::json::Json;

type Cache = Vec<data::SymbolItem>;

#[get("/symbols")]
async fn symbols(cache_mutex: &State<Mutex<Cache>>) -> Option<Json<Vec<String>>> {
    // Lock for a short period only to inspect if there is a cache
    let in_cache = {
        let cache = cache_mutex.lock().unwrap();
        (*cache).len() > 0
    };
    // Fetch symbols only if no cache was found
    let fetched_symbols: Option<Cache> = {
        if !in_cache {
            println!("No cache");
            Some(data::get_symbols().await.ok()?)
        } else {
            println!("Cache");
            None
        }
    };
    // Update the cache with the symbols we just fetch (if we did)
    let mut cache = cache_mutex.lock().unwrap();
    if let Some(new_cache) = fetched_symbols {
        *cache = new_cache;
    }

    // Build the respone
    let symbols_codes = (*cache)
        .iter()
        .map(|symb| String::clone(&symb.symbol))
        .collect();
    Some(Json(symbols_codes))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::<Cache>::new(vec!()))
        .mount("/api", routes![symbols])
}