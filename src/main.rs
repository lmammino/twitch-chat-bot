use client::PrivMessageListenerT;
use futures_util::future::BoxFuture;
use parser::PrivMsg;
use websocket_lite::Result;

use crate::client::{Client, MySender};

mod client;
mod parser;

struct Reply {}

impl PrivMessageListenerT for Reply {
    fn on_priv_msg<'s, 'p, 'b, 'sender, 'output>(&'s self, msg: &'p PrivMsg<'b>, sender: &'sender mut MySender) -> BoxFuture<'output, ()>
    where
            'p: 's,
            'b: 's,
            'sender: 's,
            'sender: 'output
    {
        Box::pin(async move {
            sender.send_text("Hello!").await.unwrap();
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let twitch_token = std::env::var("TWITCH_TOKEN").expect("Missing TWITCH_TOKEN");
    let mut client = Client::new("loige".to_string(), "loigebot".to_string());

    client.add_priv_msg_listener(Box::new(Reply {}));

    let client = client.connect(twitch_token).await?;
    client.run().await?;

    println!("Exiting the main loop");

    Ok(())
}
