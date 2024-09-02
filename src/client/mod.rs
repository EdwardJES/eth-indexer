use alloy::rpc::client::{ClientBuilder, ReqwestClient};
use poller::PollChannel;
use reqwest::Url;
pub mod poller;

pub struct Client(ReqwestClient);

impl Client {
    pub fn new(provider_string: &str) -> Self {
        // Instantiate a new client over a HTTP transport.
        let client = ClientBuilder::default().http(Url::parse(provider_string).unwrap());
        Self(client)
    }

    pub fn poll_blocks(&self) -> PollChannel {
        poller::Poller::new(self.0.clone()).spawn()
    }
}
