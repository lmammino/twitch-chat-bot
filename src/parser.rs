use std::collections::HashMap;

use nom::bytes::complete::{tag, take_till, take_until};
use nom::combinator::opt;
use nom::multi::separated_list1;
use nom::sequence::tuple;
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
        tags: HashMap<&'a str, &'a str>,
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

fn parse_key_values(msg: &str) -> IResult<&str, (&str, &str)> {
    // badge-info=;badges=vip/1
    let (msg, key) = take_until("=")(msg)?;
    let (msg, _) = tag("=")(msg)?;
    let (msg, value) = take_till(|c| c == ' ' || c == ';')(msg)?;

    Ok((msg, (key, value)))
}

fn parse_tags(msg: &str) -> IResult<&str, HashMap<&str, &str>> {
    let (msg, _) = tag("@")(msg)?;
    let (msg, kv_pairs) = separated_list1(tag(";"), parse_key_values)(msg)?;

    Ok((msg, kv_pairs.into_iter().collect::<HashMap<&str, &str>>()))
}

fn parse_priv_msg(msg: &str) -> IResult<&str, Msg<'_>> {
    // :gambuzzi!gambuzzi@gambuzzi.tmi.twitch.tv PRIVMSG #loige :something
    let (msg, tags) = opt(tuple((parse_tags, tag(" "))))(msg)?;

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
            // tags: tags.unwrap_or_else(|| (HashMap::new(), "")).0,
            tags: tags.map(|x| x.0).unwrap_or_default(),
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
                tags: HashMap::new(),
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

    #[test]
    fn test_parse_priv_msg_with_tags() {
        let msg = "@badge-info=subscriber/27;badges=broadcaster/1,subscriber/0;client-nonce=0ad2f4543a7c7ebd3814ceca0ce71434;color=#0000FF;display-name=Loige;emotes=305954156:19-26/emotesv2_0a2141aad734442b8d59867381606ff2:31-39;first-msg=0;flags=;id=ded9c28a-84d6-4b0b-b22f-1b99ca5085e3;mod=0;returning-chatter=0;room-id=442728198;subscriber=1;tmi-sent-ts=1695057155940;turbo=0;user-id=442728198;user-type= :loige!loige@loige.tmi.twitch.tv PRIVMSG #loige :hello, do you like PogChamp or loigeCrab ?";
        let msg = parse_msg(msg);
        let expected_tags: HashMap<&str, &str> = [
            ("flags", ""),
            ("mod", "0"),
            (
                "emotes",
                "305954156:19-26/emotesv2_0a2141aad734442b8d59867381606ff2:31-39",
            ),
            ("badge-info", "subscriber/27"),
            ("first-msg", "0"),
            ("returning-chatter", "0"),
            ("tmi-sent-ts", "1695057155940"),
            ("room-id", "442728198"),
            ("color", "#0000FF"),
            ("client-nonce", "0ad2f4543a7c7ebd3814ceca0ce71434"),
            ("display-name", "Loige"),
            ("subscriber", "1"),
            ("turbo", "0"),
            ("user-id", "442728198"),
            ("user-type", ""),
            ("id", "ded9c28a-84d6-4b0b-b22f-1b99ca5085e3"),
            ("badges", "broadcaster/1,subscriber/0"),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            msg,
            Msg::PrivMsg {
                nick: "loige",
                canonical_nick: "loige@loige.tmi.twitch.tv",
                msg: "hello, do you like PogChamp or loigeCrab ?",
                channel: "loige",
                tags: expected_tags,
            }
        );
    }

    #[test]
    fn test_parse_tags() {
        let msg = "@badge-info=subscriber/27;badges=broadcaster/1,subscriber/0;client-nonce=0ad2f4543a7c7ebd3814ceca0ce71434;color=#0000FF;display-name=Loige;emotes=305954156:19-26/emotesv2_0a2141aad734442b8d59867381606ff2:31-39;first-msg=0;flags=;id=ded9c28a-84d6-4b0b-b22f-1b99ca5085e3;mod=0;returning-chatter=0;room-id=442728198;subscriber=1;tmi-sent-ts=1695057155940;turbo=0;user-id=442728198;user-type= ";
        let (_, hash_map) = parse_tags(msg).unwrap();
        let expected: HashMap<&str, &str> = [
            ("flags", ""),
            ("mod", "0"),
            (
                "emotes",
                "305954156:19-26/emotesv2_0a2141aad734442b8d59867381606ff2:31-39",
            ),
            ("badge-info", "subscriber/27"),
            ("first-msg", "0"),
            ("returning-chatter", "0"),
            ("tmi-sent-ts", "1695057155940"),
            ("room-id", "442728198"),
            ("color", "#0000FF"),
            ("client-nonce", "0ad2f4543a7c7ebd3814ceca0ce71434"),
            ("display-name", "Loige"),
            ("subscriber", "1"),
            ("turbo", "0"),
            ("user-id", "442728198"),
            ("user-type", ""),
            ("id", "ded9c28a-84d6-4b0b-b22f-1b99ca5085e3"),
            ("badges", "broadcaster/1,subscriber/0"),
        ]
        .into_iter()
        .collect();
        assert_eq!(hash_map, expected);
    }
}
