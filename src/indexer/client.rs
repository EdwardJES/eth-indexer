use alloy::rpc::{
    client::{ClientBuilder, ReqwestClient},
    types::Block,
};
use futures::Stream;
use reqwest::Url;

use super::poller;

pub struct Client(ReqwestClient);

impl Client {
    pub fn new(provider_string: &str) -> Self {
        // Instantiate a new client over a HTTP transport.
        let client = ClientBuilder::default().http(Url::parse(provider_string).unwrap());
        Self(client)
    }

    pub fn block_stream(&self, interval: u64) -> impl Stream<Item = Block> + Unpin {
        poller::Poller::new(self.0.clone(), interval)
            .spawn()
            .into_stream()
    }
}
