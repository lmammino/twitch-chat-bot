use nom::IResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MsgType<'a> {
    PrivMsg, // :gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something
    Ping,    // PING :tmi.twitch.tv
    Join,    // :01ella!01ella@01ella.tmi.twitch.tv JOIN #loige
    Part,    // :zkeey!zkeey@zkeey.tmi.twitch.tv PART #loige
    Other(&'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Msg<'a> {
    pub nick: &'a str,
    pub canonical_nick: &'a str,
    pub msg: &'a str,
    pub channel: &'a str,
    pub msg_type: MsgType<'a>,
}

fn parse_priv_msg(msg: &str) -> IResult<&str, Msg<'_>> {
    // :gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something
    let (msg, _) = nom::bytes::complete::tag(":")(msg)?;
    let (msg, nick) = nom::bytes::complete::take_until("!")(msg)?;
    let (msg, _) = nom::bytes::complete::tag("!")(msg)?;
    let (msg, canonical_nick) = nom::bytes::complete::take_until(" ")(msg)?;
    let (msg, _) = nom::bytes::complete::tag(" PRIVMSG #")(msg)?;
    let (msg, channel) = nom::bytes::complete::take_until(" :")(msg)?;
    let (msg, _) = nom::bytes::complete::tag(" :")(msg)?;

    let empty = "";
    Ok((
        empty,
        Msg {
            nick,
            canonical_nick,
            msg,
            channel,
            msg_type: MsgType::PrivMsg,
        },
    ))
}

// TODO: implement parsers for the other types of messages

fn parse_msg(msg: &str) -> Msg<'_> {
    // TODO: use a nom combinator including all the possible types
    // of messages, default to "other" if it fails
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_priv_msg() {
        let msg = ":gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something";
        let (_, msg) = parse_priv_msg(msg).unwrap();
        assert_eq!(
            msg,
            Msg {
                nick: "gambuzzi",
                canonical_nick: "gambuzzi@gambuzzi.tmi.twitch.tv",
                msg: "something",
                channel: "loige",
                msg_type: MsgType::PrivMsg,
            }
        );
    }
}
