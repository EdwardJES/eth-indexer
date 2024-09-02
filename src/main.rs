use ethers::types::U64;
use futures::StreamExt;

mod client;

#[tokio::main]
async fn main() {
    let provider_str = "https://rpc.testnet.immutable.com/";
    let client = client::Client::new(provider_str);

    let channel = client.poll_blocks();
    let mut stream = channel.into_stream();

    // consume stream
    while let Some(response) = stream.next().await {
        println!("{:?}", response);
    }
}
