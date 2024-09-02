use alloy::eips::BlockId;
use alloy::rpc::client::ReqwestClient;
use alloy::rpc::types::Block;
use alloy::transports::RpcError;
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
pub struct Poller {
    /// Client to poll
    client: ReqwestClient,

    /// Config
    channel_size: usize,
    poll_interval: Duration,
    limit: usize,
    max_retries: usize,
}

/// 'static lifetime is used here as the poller will be running for the lifetime of the application.
/// This avoids lifetime issues where the params might not live as long as the poller or vice versa.
impl Poller {
    /// Create a new poller
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client,
            channel_size: 16,
            poll_interval: Duration::from_secs(1), // TODO make configurable
            limit: usize::MAX,
            max_retries: 3,
        }
    }

    pub fn spawn(self) -> PollChannel {
        let (tx, rx) = broadcast::channel::<Block>(self.channel_size);
        tokio::spawn(async move {
            let mut retries = self.max_retries;
            'outer: for _ in 0..self.limit {
                loop {
                    match self
                        .client
                        .request("eth_getBlockByNumber", (BlockId::latest(), false))
                        .await
                    {
                        Ok(resp) => {
                            if tx.send(resp).is_err() {
                                break 'outer;
                            }
                        }
                        Err(RpcError::Transport(err)) if retries > 0 && err.recoverable() => {
                            retries -= 1;
                            continue;
                        }
                        Err(err) => {
                            eprintln!("RECEIVED: Error: {:?}", err);
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

    // pub fn into_stream(self) -> impl Stream<Item = Block> + Unpin {
    //     self.spawn().into_stream()
    // }
}

#[derive(Debug)]
pub struct PollChannel {
    rx: broadcast::Receiver<Block>,
}

impl From<broadcast::Receiver<Block>> for PollChannel {
    fn from(value: broadcast::Receiver<Block>) -> Self {
        Self { rx: value }
    }
}

impl PollChannel {
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
    pub fn into_stream(self) -> impl Stream<Item = Block> + Unpin {
        let broadcast_stream: BroadcastStream<Block> = self.rx.into();
        broadcast_stream.filter_map(|r| futures::future::ready(r.ok()))
    }
}
