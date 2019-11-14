use tokio;

use wamp_client;

#[tokio::main]
async fn main() {
    wamp_client::start().await;
}
