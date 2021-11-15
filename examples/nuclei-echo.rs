use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::prelude::*;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(
        feature = "async-tls",
        feature = "tokio-native-tls",
        feature = "tokio-openssl"
    ))]
    let url = "wss://ws.ifelse.io";
    #[cfg(not(any(
        feature = "async-tls",
        feature = "tokio-native-tls",
        feature = "tokio-openssl"
    )))]
    let url = "ws://ws.ifelse.io";

    let (mut ws_stream, _) = connect_async(url).await?;

    let text = "Hello, World!";

    println!("Sending: \"{}\"", text);
    ws_stream.send(Message::text(text)).await?;

    let _msg = ws_stream.next().await.ok_or("didn't receive anything")??;
    let msg = ws_stream.next().await.ok_or("didn't receive anything")??;

    println!("Received: {:?}", msg);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run())
}
