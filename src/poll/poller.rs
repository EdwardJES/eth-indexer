use alloy_json_rpc::{RpcParam, RpcReturn};
use alloy_rpc_client::ReqwestClient;
use futures::stream::Stream;
use futures::StreamExt;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

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
    max_retries: usize,

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
            max_retries: 3,
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

    pub fn spawn(self) -> PollChannel<Resp> {
        let (tx, rx) = broadcast::channel::<Resp>(self.channel_size);
        tokio::spawn(async move {
            let params = self.params;
            let mut retries = self.max_retries;
            'outer: for _ in 0..self.limit {
                loop {
                    match self
                        .client
                        .request(self.method.clone(), params.clone())
                        .await
                    {
                        Ok(resp) => {
                            if tx.send(resp).is_err() {
                                break 'outer;
                            }
                        }
                        Err(alloy_json_rpc::RpcError::Transport(err))
                            if retries > 0 && err.recoverable() =>
                        {
                            retries -= 1;
                            continue;
                        }
                        Err(err) => {
                            eprintln!("Error: {:?}", err);
                            break 'outer;
                        }
                    }

                    break;
                }
                println!("Sleeping..");
                tokio::time::sleep(self.poll_interval).await;
            }
        });

        rx.into()
    }

    pub fn into_stream(self) -> impl Stream<Item = Resp> + Unpin {
        self.spawn().into_stream()
    }
}

#[derive(Debug)]
pub struct PollChannel<Resp> {
    rx: broadcast::Receiver<Resp>,
}

impl<Resp> From<broadcast::Receiver<Resp>> for PollChannel<Resp> {
    fn from(value: broadcast::Receiver<Resp>) -> Self {
        Self { rx: value }
    }
}

impl<Resp> Deref for PollChannel<Resp> {
    type Target = broadcast::Receiver<Resp>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<Resp> DerefMut for PollChannel<Resp> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

impl<Resp> PollChannel<Resp>
where
    Resp: RpcReturn + Clone,
{
    pub fn resubscribe(&self) -> Self {
        Self {
            rx: self.rx.resubscribe(),
        }
    }

    /// Convert the channel into a stream.
    ///
    /// This will first convert the broadcast receiver into a broadcast stream.
    /// This is simply a wrapper that implements steam.
    ///
    /// The stream is then mapped to return Options instead of Results.
    /// This creates a nicer API for the consumer, as the stream is long living and will ignore errors.
    pub fn into_stream(self) -> impl Stream<Item = Resp> + Unpin {
        let broadcast_stream: BroadcastStream<Resp> = self.rx.into();
        broadcast_stream.filter_map(|r| futures::future::ready(r.ok()))
    }
}
