use dotenv;
use futures::StreamExt;
use indexer::Indexer;
mod indexer;

#[tokio::main]
async fn main() {
    let provider_str = dotenv::var("PROVIDER_URL").expect("PROVIDER must be set");
    let indexer = Indexer::new(&provider_str);

    let interval = 2;
    let mut stream = indexer.client.block_stream(interval);

    // consume stream
    while let Some(response) = stream.next().await {
        println!("{:#?}", response);
    }
}
