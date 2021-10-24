use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

mod common;
mod deserialize_weird_floats;
mod ishares;
use common::{Error, FundManager, ETF};
use ishares::Ishare;

pub struct AvailableETFs {
    etf_to_manager: HashMap<String, Arc<Mutex<dyn FundManager>>>,
}

impl AvailableETFs {
    pub async fn new() -> AvailableETFs {
        let mut etf_to_manager = HashMap::<String, Arc<Mutex<dyn FundManager>>>::new();

        let manager_constructors = [Ishare::new];
        for manager_constructor in manager_constructors.iter() {
            match manager_constructor().await {
                Ok(manager) => {
                    let etfs = manager.etfs_under_management();
                    let manager_ref = Arc::new(Mutex::new(manager));
                    for etf in etfs {
                        etf_to_manager.insert(etf, manager_ref.clone());
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }

        AvailableETFs { etf_to_manager }
    }

    pub fn etf_list(&self) -> Vec<String> {
        self.etf_to_manager.keys().map(|s| s.clone()).collect()
    }

    pub async fn etf_details(&self, ticker: String) -> Result<ETF, Error> {
        let mut manager = self
            .etf_to_manager
            .get(&ticker)
            .ok_or("ETF not found.")?
            .lock()
            .await;
        manager.etf_details(&ticker).await
    }
}
