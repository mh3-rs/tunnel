//! The quest bots' tunnel (mh3sp-rs: `roadmap/experiment-bots.md`, `architecture.md` decision 11): the server is the
//! bots' body, the brain (`mh3sp-brain`) decides what they do. One TCP connection carries every bot, in
//! length-prefixed frames, each a message encoded in binary (postcard). A bot is named by its Capcom ID; the party's
//! events name the party. Only the messages built so far: the rest of the protocol (behaviours, the party's events,
//! attacks) comes with the sprints that use it.

use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio_util::codec::LengthDelimitedCodec;

/// A party, as the server knows it: world server, gate, city, quest slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Party {
    pub server: u32,
    pub gate: u16,
    pub city: u16,
    pub circle: u32,
}

/// From the server: commands.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToBrain {
    /// A bot (its Capcom ID) is now a member of this party.
    Create { bot: String, party: Party },
    /// The bot left its party (asked, or the party ended).
    Leave { bot: String },
    /// In a quest: where a hunter of the party is (the game's coordinates and area), about twice a second. Every bot
    /// of the party reports it: the latest one is the truth.
    Hunter {
        party: Party,
        hunter: String,
        position: [f32; 3],
        area: u8,
    },
}

/// From the brain: what a bot does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToServer {
    /// Walk to a position (the game's coordinates).
    Move { bot: String, position: [f32; 3] },
    /// Enter an area of the quest's stage.
    ChangeArea { bot: String, area: u8 },
}

/// The frames: a length, then the message.
pub fn codec() -> LengthDelimitedCodec {
    LengthDelimitedCodec::builder()
        .max_frame_length(64 * 1024)
        .new_codec()
}

pub fn encode<T: Serialize>(message: &T) -> Result<Bytes> {
    Ok(postcard::to_stdvec(message)?.into())
}

pub fn decode<'a, T: Deserialize<'a>>(frame: &'a [u8]) -> Result<T> {
    Ok(postcard::from_bytes(frame)?)
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use super::*;

    #[test]
    fn messages_round_trip_through_the_frames() {
        let party = Party {
            server: 1,
            gate: 2,
            city: 3,
            circle: 1,
        };
        let down = [
            ToBrain::Create {
                bot: "0ABCDE".into(),
                party,
            },
            ToBrain::Leave {
                bot: "0ABCDE".into(),
            },
            ToBrain::Hunter {
                party,
                hunter: "ERESG5".into(),
                position: [-1025.0, 263.0, 640.0],
                area: 1,
            },
        ];
        let up = [
            ToServer::Move {
                bot: "0ABCDE".into(),
                position: [-1025.0, 263.0, 640.0],
            },
            ToServer::ChangeArea {
                bot: "0ABCDE".into(),
                area: 3,
            },
        ];
        let mut codec = codec();
        let mut wire = BytesMut::new();
        for message in &down {
            codec.encode(encode(message).unwrap(), &mut wire).unwrap();
        }
        for message in &up {
            codec.encode(encode(message).unwrap(), &mut wire).unwrap();
        }
        for message in &down {
            let frame = codec.decode(&mut wire).unwrap().unwrap();
            assert_eq!(&decode::<ToBrain>(&frame).unwrap(), message);
        }
        for message in &up {
            let frame = codec.decode(&mut wire).unwrap().unwrap();
            assert_eq!(&decode::<ToServer>(&frame).unwrap(), message);
        }
        assert!(codec.decode(&mut wire).unwrap().is_none());
    }

    #[test]
    fn garbage_is_an_error_not_a_panic() {
        assert!(decode::<ToBrain>(&[0xff, 0xff, 0xff]).is_err());
    }
}
