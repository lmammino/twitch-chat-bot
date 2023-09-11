use nom::bytes::complete::{tag, take_until};
use nom::{branch::alt, IResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Msg<'a> {
    // :gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something
    #[allow(clippy::enum_variant_names)]
    PrivMsg {
        nick: &'a str,
        canonical_nick: &'a str,
        msg: &'a str,
        channel: &'a str,
    },
    // PING :tmi.twitch.tv
    Ping {
        server_name: &'a str,
    },
    // :01ella!01ella@01ella.tmi.twitch.tv JOIN #loige
    Join {
        nick: &'a str,
        canonical_nick: &'a str,
        channel: &'a str,
    },
    // :zkeey!zkeey@zkeey.tmi.twitch.tv PART #loige
    Part {
        nick: &'a str,
        canonical_nick: &'a str,
        channel: &'a str,
    },
    Other {
        msg: &'a str,
    },
}

fn parse_nick(msg: &str) -> IResult<&str, (&str, &str)> {
    let (msg, _) = tag(":")(msg)?;
    let (msg, nick) = take_until("!")(msg)?;
    let (msg, _) = tag("!")(msg)?;
    let (remainder, canonical_nick) = take_until(" ")(msg)?;
    Ok((remainder, (nick, canonical_nick)))
}

fn parse_other(msg: &str) -> IResult<&str, Msg<'_>> {
    Ok(("", Msg::Other { msg }))
}

fn parse_join_or_part(msg: &str) -> IResult<&str, Msg<'_>> {
    // :01ella!01ella@01ella.tmi.twitch.tv JOIN #loige
    let (msg, (nick, canonical_nick)) = parse_nick(msg)?;
    let (channel, action) = alt((tag(" JOIN #"), tag(" PART #")))(msg)?;

    if action == " PART #" {
        return Ok((
            "",
            Msg::Part {
                nick,
                canonical_nick,
                channel,
            },
        ));
    }

    Ok((
        "",
        Msg::Join {
            nick,
            canonical_nick,
            channel,
        },
    ))
}

fn parse_ping(msg: &str) -> IResult<&str, Msg<'_>> {
    // PING :tmi.twitch.tv
    let (server_name, _) = tag("PING :")(msg)?;
    Ok(("", Msg::Ping { server_name }))
}

fn parse_priv_msg(msg: &str) -> IResult<&str, Msg<'_>> {
    // :gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something
    let (msg, (nick, canonical_nick)) = parse_nick(msg)?;
    let (msg, _) = tag(" PRIVMSG #")(msg)?;
    let (msg, channel) = take_until(" :")(msg)?;
    let (msg, _) = tag(" :")(msg)?;

    let empty = "";
    Ok((
        empty,
        Msg::PrivMsg {
            nick,
            canonical_nick,
            msg,
            channel,
        },
    ))
}

pub fn parse_msg(input: &str) -> Msg<'_> {
    // it's ok to unwrap because if we fail we end up in "other"
    let (_remaining, msg) =
        alt((parse_ping, parse_priv_msg, parse_join_or_part, parse_other))(input).unwrap();
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_priv_msg() {
        let msg = ":gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something";
        let msg = parse_msg(msg);
        assert_eq!(
            msg,
            Msg::PrivMsg {
                nick: "gambuzzi",
                canonical_nick: "gambuzzi@gambuzzi.tmi.twitch.tv",
                msg: "something",
                channel: "loige",
            }
        );
    }

    #[test]
    fn test_parse_ping() {
        let msg = "PING :tmi.twitch.tv";
        let msg = parse_msg(msg);
        assert_eq!(
            msg,
            Msg::Ping {
                server_name: "tmi.twitch.tv",
            }
        );
    }

    #[test]
    fn test_parse_join() {
        let msg = ":01ella!01ella@01ella.tmi.twitch.tv JOIN #loige";
        let msg = parse_msg(msg);
        assert_eq!(
            msg,
            Msg::Join {
                nick: "01ella",
                canonical_nick: "01ella@01ella.tmi.twitch.tv",
                channel: "loige",
            }
        );
    }

    #[test]
    fn test_parse_part() {
        let msg = ":01ella!01ella@01ella.tmi.twitch.tv PART #loige";
        let msg = parse_msg(msg);
        assert_eq!(
            msg,
            Msg::Part {
                nick: "01ella",
                canonical_nick: "01ella@01ella.tmi.twitch.tv",
                channel: "loige",
            }
        );
    }

    #[test]
    fn test_parse_other() {
        let msg = ":loige.tmi.twitch.tv 353 loige = #loige :loige";
        let msg = parse_msg(msg);
        assert_eq!(
            msg,
            Msg::Other {
                msg: ":loige.tmi.twitch.tv 353 loige = #loige :loige"
            }
        );
    }
}
