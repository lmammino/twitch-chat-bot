use crate::parser::{parse_msg, Msg};
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use websocket_lite::{Message, Opcode, Result};

mod parser;

#[tokio::main]
async fn main() -> Result<()> {
    let twitch_token = std::env::var("TWITCH_TOKEN").expect("Missing TWITCH_TOKEN");

    let builder = websocket_lite::ClientBuilder::new("wss://irc-ws.chat.twitch.tv:443")?;
    let mut ws_stream = builder.async_connect().await?;

    // Send capabilities requests
    let cap_payload = "CAP REQ :twitch.tv/membership";
    ws_stream.send(Message::text(cap_payload)).await?;

    // Send auth details
    let auth_payload = format!("PASS oauth:{}", twitch_token);
    ws_stream.send(Message::text(auth_payload)).await?;
    // let msg = ws_stream.next().await;
    // println!("AUTH RESPONSE: {:?}", msg);

    // Send nickname
    let nick_payload = "NICK loigebot";
    ws_stream.send(Message::text(nick_payload)).await?;
    // let msg = ws_stream.next().await;
    // println!("NICK RESPONSE: {:?}", msg);

    // Join channel
    let join_payload = "JOIN #loige";
    ws_stream.send(Message::text(join_payload)).await?;

    while let Some(msg) = ws_stream.next().await {
        if let Ok(m) = msg {
            match m.opcode() {
                Opcode::Text => {
                    let text = m.as_text().unwrap();
                    for line in text.lines() {
                        let msg = parse_msg(line);

                        println!("MESSAGE RECEIVED: {:?}", msg);
                        match msg {
                            Msg::Ping { server_name } => {
                                let pong_payload = format!("PONG :{}", server_name);
                                ws_stream.send(Message::text(pong_payload)).await?;
                            }
                            Msg::Join {
                                nick,
                                canonical_nick: _canonical_nick,
                                channel,
                            } => {
                                ws_stream
                                    .send(Message::text(format!(
                                        "PRIVMSG #{} :Hi @{}!",
                                        channel, nick
                                    )))
                                    .await?;
                            }
                            _ => {
                                // TODO: implement additional behaviours if needed
                            }
                        }
                    }
                }
                Opcode::Ping => ws_stream.send(Message::pong(m.into_data())).await?,
                Opcode::Close => {
                    println!("Received close message");
                    let _ = ws_stream.send(Message::close(None)).await;
                    break;
                }
                Opcode::Pong | Opcode::Binary => {}
            }
        } else {
            println!("Error reading message: {:?}", msg);
            let _ = ws_stream.send(Message::close(None)).await;
            break;
        }
    }

    println!("Exiting the main loop");

    Ok(())
}
