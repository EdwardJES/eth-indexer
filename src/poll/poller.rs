use alloy_json_rpc::{RpcParam, RpcReturn};
use alloy_rpc_client::ReqwestClient;
use std::borrow::Cow;
use std::clone;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Debug)]
#[must_use = "`spawn` or `into_stream` must be used to run the poller"]
pub struct Poller<Params, Resp> {
    /// Client to poll
    client: ReqwestClient,

    /// Method
    method: Cow<'static, str>,
    params: Params,

    /// Config
    channel_size: usize,
    poll_interval: Duration,
    limit: usize,

    _pd: PhantomData<fn() -> Resp>,
}

/// 'static lifetime is used here as the poller will be running for the lifetime of the application.
/// This avoids lifetime issues where the params might not live as long as the poller or vice versa.
impl<Params, Resp> Poller<Params, Resp>
where
    Params: RpcParam + 'static,
    Resp: RpcReturn + Clone,
{
    /// Create a new poller
    pub fn new(
        client: ReqwestClient,
        method: impl Into<Cow<'static, str>>,
        params: Params,
        poll_interval: Duration,
    ) -> Self {
        Self {
            client,
            method: method.into(),
            params,
            channel_size: 16,
            poll_interval,
            limit: usize::MAX,
            _pd: PhantomData,
        }
    }

    pub const fn channel_size(&self) -> usize {
        self.channel_size
    }

    pub fn set_channel_size(&mut self, channel_size: usize) {
        self.channel_size = channel_size;
    }

    pub fn with_channel_size(mut self, channel_size: usize) -> Self {
        self.set_channel_size(channel_size);
        self
    }

    pub const fn limit(&self) -> usize {
        self.limit
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.set_limit(limit);
        self
    }

    pub const fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    pub fn set_poll_interval(&mut self, poll_interval: Duration) {
        self.poll_interval = poll_interval;
    }

    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.set_poll_interval(poll_interval);
        self
    }

    pub fn spawn()
}
