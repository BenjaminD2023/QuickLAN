//! Bounded messages over an already OS-authenticated, private local stream.
//! Never expose this protocol on a TCP socket or accept arbitrary core config.
use crate::{adapter::Peer, error::Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};
use zeroize::Zeroizing;

pub const MAX_FRAME: usize = 1_048_576;
pub const VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum EngineReply {
    Ready {
        protocol: u8,
        engine_version: String,
    },
    State {
        virtual_ip: Option<String>,
        peers: Vec<Peer>,
    },
    Stopped {},
    Failed {
        code: Error,
    },
}

/// Length is checked before allocation. Plaintext buffers are erased on drop.
pub fn read_frame(reader: &mut impl Read, maximum: usize) -> io::Result<Zeroizing<Vec<u8>>> {
    let mut header = [0; 4];
    reader.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 || len > maximum.min(MAX_FRAME) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid frame length",
        ));
    }
    let mut bytes = Zeroizing::new(vec![0; len]);
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn receive<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    let bytes = read_frame(reader, MAX_FRAME)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid message"))
}

pub fn send(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let bytes = Zeroizing::new(
        serde_json::to_vec(value)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid message"))?,
    );
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Message too large",
        ));
    }
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_and_truncated_input_before_parsing() {
        assert!(read_frame(&mut &u32::MAX.to_be_bytes()[..], MAX_FRAME).is_err());
        assert!(read_frame(&mut &[0, 0, 0, 5, 1, 2][..], MAX_FRAME).is_err());
        assert!(read_frame(&mut &[0, 0, 0, 0][..], MAX_FRAME).is_err());
    }
    #[test]
    fn preserves_frame_boundaries_and_rejects_unknown_fields() {
        let mut bytes = vec![];
        send(&mut bytes, &EngineReply::Stopped {}).unwrap();
        send(
            &mut bytes,
            &EngineReply::Failed {
                code: Error::CoreFailed,
            },
        )
        .unwrap();
        let mut reader = &bytes[..];
        assert!(matches!(
            receive(&mut reader).unwrap(),
            EngineReply::Stopped {}
        ));
        assert!(matches!(
            receive(&mut reader).unwrap(),
            EngineReply::Failed {
                code: Error::CoreFailed
            }
        ));
        assert!(reader.is_empty());
        assert!(
            serde_json::from_str::<EngineReply>(r#"{"event":"stopped","command":"anything"}"#)
                .is_err()
        );
    }
}
