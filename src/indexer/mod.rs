pub mod client;
pub mod poller;
pub mod store;
use client::Client;
use store::Store;
pub struct Indexer {
    pub client: Client,
    pub store: Store,
}

impl Indexer {
    pub fn new(provider_string: &str) -> Self {
        let client = Client::new(provider_string);
        let mut store = Store::new("indexer.db");
        store.create_tables();
        Self { client, store }
    }
}
