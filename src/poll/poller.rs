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

    _pd: PhantomData<fn() -> Resp>,
}

/// 'static lifetime is used here as the poller will be running for the lifetime of the application.
/// This avoids lifetime issues where the params might not live as long as the poller or vice versa.
impl<Params, Resp> Poller<Params, Resp>
where
    Params: RpcParam + 'static,
    Resp: RpcReturn + Clone,
{
}
