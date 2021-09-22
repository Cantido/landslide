// SPDX-FileCopyrightText: 2021 Rosa Richter
//
// SPDX-License-Identifier: MIT

//! Implementation of the Peer Wire Protocol (PWP).

use bitflags::bitflags;
use bytes::{Bytes, BytesMut, BufMut};
use std::convert::TryInto;

#[derive(Debug, Default)]
pub struct Connection {
    us_choking: ChokeFlag,
    us_interested: InterestFlag,
    them_choking: ChokeFlag,
    them_interested: InterestFlag,
}

#[derive(Debug)]
pub enum InterestFlag {
    Interested,
    NotInterested,
}

impl Default for InterestFlag {
    fn default() -> Self {
        InterestFlag::NotInterested
    }
}

#[derive(Debug)]
pub enum ChokeFlag {
    Choked,
    Unchoked,
}

impl Default for ChokeFlag {
    fn default() -> Self {
        ChokeFlag::Choked
    }
}

/// The establishing handshake that starts a PWP connection.
pub struct Handshake {
    flags: HandshakeFlags,
    info_hash: crate::InfoHash,
    peer_id: crate::PeerId,
}

bitflags! {
/// The reserved bits of the handshake, used to flag certain extensions.
    struct HandshakeFlags: u64 {
        const FAST = 0x0000_0000_0000_0400;
        const DHT = 0x0000_0000_0000_0001;
        const EXTENDED = 0x0000_0000_1000_0000;
    }
}
impl Handshake {
    /// Write this handshake to a writer.
    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(68);
        buf.put_u8(19);
        buf.put(&b"BitTorrent Protocol"[..]);
        buf.put_u64(self.flags.bits());
        buf.put(&self.peer_id[..]);
        buf.put(&self.info_hash[..]);
        buf.freeze()
    }
}

/// Messages sent over PWP after the handshake.
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    Uninterested,
    Have(crate::PieceIndex),
    Bitfield(Bytes),
    Request {
        index: crate::PieceIndex,
        offset: crate::BlockOffset,
        length: u32,
    },
    Cancel {
        index: crate::PieceIndex,
        offset: crate::BlockOffset,
        length: u32,
    },
    Block {
        index: crate::PieceIndex,
        offset: crate::BlockOffset,
        data: Bytes,
    },
}

impl Message {
    /// Serialize this message.
    pub fn serialize(self) -> Bytes {
        match self {
            Message::KeepAlive => Bytes::from(vec![0x00, 0x00, 0x00, 0x00]),
            Message::Choke => Bytes::from(vec![0x00, 0x00, 0x00, 0x01, 0x00]),
            Message::Unchoke => Bytes::from(vec![0x00, 0x00, 0x00, 0x01, 0x01]),
            Message::Interested => Bytes::from(vec![0x00, 0x00, 0x00, 0x01, 0x02]),
            Message::Uninterested => Bytes::from(vec![0x00, 0x00, 0x00, 0x01, 0x03]),
            Message::Have(index) => {
                let mut buf = BytesMut::with_capacity(9);
                buf.put_u32(5);
                buf.put_u8(0x04);
                buf.put_u32(index);
                buf.freeze()
            }
            Message::Bitfield(b) => {
                let len = b.len() + 1;
                let mut buf = BytesMut::with_capacity(len + 4);
                buf.put_u32(len.try_into().expect("Bitfield is too big to encode in a bitfield message."));
                buf.put_u8(0x05);
                buf.put(b);
                buf.freeze()
            }
            Message::Request {
                index,
                offset,
                length,
            } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u32(13);
                buf.put_u8(0x06);
                buf.put_u32(index);
                buf.put_u32(offset);
                buf.put_u32(length);
                buf.freeze()
            }
            Message::Cancel {
                index,
                offset,
                length,
            } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u32(13);
                buf.put_u8(0x08);
                buf.put_u32(index);
                buf.put_u32(offset);
                buf.put_u32(length);
                buf.freeze()
            }
            Message::Block {
                index,
                offset,
                data,
            } => {
                let len = data.len() + 9;
                let mut buf = BytesMut::with_capacity(len + 4);
                buf.put_u32(len.try_into().expect("Data is too big to fit in a block message."));
                buf.put_u8(0x07);
                buf.put_u32(index);
                buf.put_u32(offset);
                buf.put(data);
                buf.freeze()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pwp::Handshake;
    use crate::pwp::HandshakeFlags;
    use crate::pwp::Message;
    use bytes::Bytes;

    #[test]
    fn serialize_handshake() {
        let handshake = Handshake {
            flags: HandshakeFlags::FAST | HandshakeFlags::DHT,
            peer_id: *b"Landslide Experiment",
            info_hash: *b"12345678901234567890",
        };

        let buf = handshake.serialize();

        assert_eq!(buf.len(), 68);
    }

    #[test]
    fn serialize_keepalive() {
        let msg = Message::KeepAlive;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 4);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x00);
    }
    #[test]
    fn serialize_choke() {
        let msg = Message::Choke;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x00);
    }

    #[test]
    fn serialize_unchoke() {
        let msg = Message::Unchoke;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x01);
    }

    #[test]
    fn serialize_interested() {
        let msg = Message::Interested;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x02);
    }

    #[test]
    fn serialize_uninterested() {
        let msg = Message::Uninterested;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x03);
    }

    #[test]
    fn serialize_have() {
        let msg = Message::Have(23);
        let buf = msg.serialize();

        assert_eq!(buf.len(), 9);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x05);

        assert_eq!(buf[4], 0x04);

        assert_eq!(buf[5], 0x00);
        assert_eq!(buf[6], 0x00);
        assert_eq!(buf[7], 0x00);
        assert_eq!(buf[8], 23);
    }

    #[test]
    fn serialize_bitfield() {
        let msg = Message::Bitfield(Bytes::from(vec![0xFF, 0xFF, 0xFF]));
        let buf = msg.serialize();

        assert_eq!(buf.len(), 8);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x04);

        assert_eq!(buf[4], 0x05);

        assert_eq!(buf[5], 0xFF);
        assert_eq!(buf[6], 0xFF);
        assert_eq!(buf[7], 0xFF);
    }

    #[test]
    fn serialize_request() {
        let msg = Message::Request {
            index: 666,
            offset: 420,
            length: 16384,
        };
        let buf = msg.serialize();

        assert_eq!(buf.len(), 17);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x0d);

        assert_eq!(buf[4], 0x06);

        assert_eq!(buf[5], 0x00);
        assert_eq!(buf[6], 0x00);
        assert_eq!(buf[7], 0x02);
        assert_eq!(buf[8], 0x9a);

        assert_eq!(buf[9], 0x00);
        assert_eq!(buf[10], 0x00);
        assert_eq!(buf[11], 0x01);
        assert_eq!(buf[12], 0xA4);

        assert_eq!(buf[13], 0x00);
        assert_eq!(buf[14], 0x00);
        assert_eq!(buf[15], 0x40);
        assert_eq!(buf[16], 0x00);
    }

    #[test]
    fn serialize_cancel() {
        let msg = Message::Cancel {
            index: 666,
            offset: 420,
            length: 16384,
        };
        let buf = msg.serialize();

        assert_eq!(buf.len(), 17);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x0d);

        assert_eq!(buf[4], 0x08);

        assert_eq!(buf[5], 0x00);
        assert_eq!(buf[6], 0x00);
        assert_eq!(buf[7], 0x02);
        assert_eq!(buf[8], 0x9a);

        assert_eq!(buf[9], 0x00);
        assert_eq!(buf[10], 0x00);
        assert_eq!(buf[11], 0x01);
        assert_eq!(buf[12], 0xA4);

        assert_eq!(buf[13], 0x00);
        assert_eq!(buf[14], 0x00);
        assert_eq!(buf[15], 0x40);
        assert_eq!(buf[16], 0x00);
    }

    #[test]
    fn serialize_block() {
        let msg = Message::Block {
            index: 666,
            offset: 420,
            data: Bytes::from(vec![4, 8, 15, 16, 23, 42]),
        };
        let buf = msg.serialize();

        assert_eq!(buf.len(), 19);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x0f);

        assert_eq!(buf[4], 0x07);

        assert_eq!(buf[5], 0x00);
        assert_eq!(buf[6], 0x00);
        assert_eq!(buf[7], 0x02);
        assert_eq!(buf[8], 0x9a);

        assert_eq!(buf[9], 0x00);
        assert_eq!(buf[10], 0x00);
        assert_eq!(buf[11], 0x01);
        assert_eq!(buf[12], 0xA4);

        assert_eq!(buf[13], 4);
        assert_eq!(buf[14], 8);
        assert_eq!(buf[15], 15);
        assert_eq!(buf[16], 16);
        assert_eq!(buf[17], 23);
        assert_eq!(buf[18], 42);
    }
}
