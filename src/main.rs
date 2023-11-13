use parser::PrivMsg;
use websocket_lite::Result;

use crate::client::Client;

mod client;
mod parser;

#[tokio::main]
async fn main() -> Result<()> {
    let twitch_token = std::env::var("TWITCH_TOKEN").expect("Missing TWITCH_TOKEN");
    let mut client = Client::new("loige".to_string(), "loigebot".to_string());

    client.add_priv_msg_listener(Box::new(|m: PrivMsg<'_>, connected_client| {
        // let runtime = tokio::runtime::Runtime::new().unwrap();
        // println!("PRIV MSG: {:?}", m);
        // TODO: the code below does not work because we end up borrrowing mutably more than once! We probably need a Mutex!
        return Box::new(async {
            connected_client
                .send_msg(format!("Hello {}! How are you?", m.user.nick).as_str())
                .await;
        });
    }));

    let client = client.connect(twitch_token).await?;
    client.run().await?;

    println!("Exiting the main loop");

    Ok(())
}
