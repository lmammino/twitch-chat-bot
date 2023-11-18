use client::PrivMessageListenerT;
use futures_util::future::BoxFuture;
use parser::PrivMsg;
use websocket_lite::Result;

use crate::client::{Client, MySender};

mod client;
mod parser;

struct Reply {}

impl PrivMessageListenerT for Reply {
    fn on_priv_msg<'sel, 'msg, 'body, 'sender, 'output>(&'sel self, msg: &'msg PrivMsg<'body>, sender: &'sender mut MySender) -> BoxFuture<'output, ()>
    where
            'msg: 'sel,
            'body: 'sel,
            'sender: 'sel,
            'sender: 'output,
            'msg: 'output,
            'body: 'output
    {
        Box::pin(async move {
            sender.send_text(format!("Hello {}! How are you?", msg.user.nick).as_str()).await.unwrap();
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
