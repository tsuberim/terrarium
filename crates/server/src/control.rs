//! External creature control — envelope wire types + session registry.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use terrarium_sim::Payload;
use tokio::sync::broadcast;

use crate::wire::serde_u64;

const CONTROL_CHANNEL_CAP: usize = 256;
const MAX_SESSIONS_PER_CREATURE: usize = 4;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeWire {
    pub kind: u32,
    pub pad: u32,
    pub words: Vec<String>,
}

impl EnvelopeWire {
    pub fn to_payload(&self) -> Result<Payload, &'static str> {
        if self.words.len() != 7 {
            return Err("invalid_envelope");
        }
        let mut words = [0u64; 7];
        for (i, w) in self.words.iter().enumerate() {
            words[i] = parse_u64_word(w)?;
        }
        Ok(payload_from_words(self.kind, self.pad, words))
    }
}

pub fn payload_to_envelope(payload: &Payload) -> EnvelopeWire {
    EnvelopeWire {
        kind: payload.tag(),
        pad: terrarium_sim::abi::read_u32(&payload.bytes, 4),
        words: payload_words(payload)
            .iter()
            .map(|w| w.to_string())
            .collect(),
    }
}

fn payload_words(payload: &Payload) -> [u64; 7] {
    [
        payload.a(),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER + 8),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER + 16),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER + 24),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER + 32),
        terrarium_sim::abi::read_u64(&payload.bytes, terrarium_sim::abi::PAYLOAD_HEADER + 40),
    ]
}

fn payload_from_words(kind: u32, pad: u32, words: [u64; 7]) -> Payload {
    let mut p = terrarium_sim::abi::payload_from_scalars(
        kind,
        (pad & 0xff) as i32,
        words[0],
        words[1],
        words[2],
        words[3],
        words[4],
        words[5],
        words[6],
    );
    terrarium_sim::abi::write_u32(&mut p.bytes, 4, pad);
    p
}

fn parse_u64_word(s: &str) -> Result<u64, &'static str> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|_| "invalid_word")
    } else {
        s.parse().map_err(|_| "invalid_word")
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlServerMsg {
    Attached {
        #[serde(with = "serde_u64")]
        creature_id: u64,
        #[serde(with = "serde_u64")]
        account_creature_id: u64,
        tick: u64,
    },
    Recv {
        #[serde(with = "serde_u64")]
        sender: u64,
        envelope: EnvelopeWire,
    },
    Detached {
        reason: String,
    },
    Error {
        code: String,
        message: String,
    },
    Pong,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlClientMsg {
    Signal {
        #[serde(with = "serde_u64")]
        target: u64,
        envelope: EnvelopeWire,
    },
    Broadcast {
        envelope: EnvelopeWire,
    },
    Ping,
}

pub struct ControlRegistry {
    channels: RwLock<HashMap<u64, broadcast::Sender<ControlServerMsg>>>,
    session_counts: RwLock<HashMap<u64, usize>>,
}

impl ControlRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            channels: RwLock::new(HashMap::new()),
            session_counts: RwLock::new(HashMap::new()),
        })
    }

    pub fn try_attach(
        &self,
        creature_id: u64,
    ) -> Result<broadcast::Receiver<ControlServerMsg>, AttachError> {
        let mut counts = self.session_counts.write();
        let entry = counts.entry(creature_id).or_insert(0);
        if *entry >= MAX_SESSIONS_PER_CREATURE {
            return Err(AttachError::TooManySessions);
        }
        *entry += 1;
        drop(counts);
        Ok(self.subscribe(creature_id))
    }

    pub fn detach(&self, creature_id: u64) {
        let mut counts = self.session_counts.write();
        if let Some(n) = counts.get_mut(&creature_id) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                counts.remove(&creature_id);
            }
        }
    }

    pub fn notify_recv(&self, creature_id: u64, sender: u64, payload: Payload) {
        let tx = self.channels.read().get(&creature_id).cloned();
        if let Some(tx) = tx {
            let _ = tx.send(ControlServerMsg::Recv {
                sender,
                envelope: payload_to_envelope(&payload),
            });
        }
    }

    pub fn notify_detached(&self, creature_id: u64, reason: &str) {
        let tx = self.channels.read().get(&creature_id).cloned();
        if let Some(tx) = tx {
            let _ = tx.send(ControlServerMsg::Detached {
                reason: reason.into(),
            });
        }
    }

    fn subscribe(&self, creature_id: u64) -> broadcast::Receiver<ControlServerMsg> {
        let mut channels = self.channels.write();
        if let Some(tx) = channels.get(&creature_id) {
            return tx.subscribe();
        }
        let (tx, rx) = broadcast::channel(CONTROL_CHANNEL_CAP);
        channels.insert(creature_id, tx);
        rx
    }
}

#[derive(Debug)]
pub enum AttachError {
    TooManySessions,
}

impl AttachError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooManySessions => "too_many_sessions",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_roundtrip() {
        let mut p = Payload::default();
        p.set_tag(8);
        p.set_a(99);
        let env = payload_to_envelope(&p);
        let back = env.to_payload().unwrap();
        assert_eq!(back.bytes, p.bytes);
    }
}
