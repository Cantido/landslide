// SPDX-FileCopyrightText: 2021 Rosa Richter
//
// SPDX-License-Identifier: MIT

use std::io::Write;

/// Messages sent over the Peer Wire Protocol (PWP)
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    Uninterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request {
        index: u32,
        offset: u32,
        length: u32,
    },
    Cancel {
        index: u32,
        offset: u32,
        length: u32,
    },
    Block {
        index: u32,
        offset: u32,
        data: Vec<u8>,
    },
}

impl Message {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        match self {
            Message::KeepAlive => writer.write_all(&[]),
            Message::Choke => writer.write_all(&[0x00, 0x00, 0x00, 0x01, 0x00]),
            Message::Unchoke => writer.write_all(&[0x00, 0x00, 0x00, 0x01, 0x01]),
            Message::Interested => writer.write_all(&[0x00, 0x00, 0x00, 0x01, 0x02]),
            Message::Uninterested => writer.write_all(&[0x00, 0x00, 0x00, 0x01, 0x03]),
            Message::Have(x) => {
              writer.write_all(&[0x00, 0x00, 0x00, 0x05, 0x04])?;
              writer.write_all(&x.to_be_bytes())
            },
            Message::Bitfield(b) => {
              let len = b.len() + 1;
              let len_bytes = (len as i32).to_be_bytes();

              writer.write_all(&len_bytes)?;
              writer.write_all(&[0x05])?;
              writer.write_all(b)
            },
            Message::Request{ index, offset, length } => {
              writer.write_all(&[0x00, 0x00, 0x00, 0x0d, 0x06])?;
              writer.write_all(&index.to_be_bytes())?;
              writer.write_all(&offset.to_be_bytes())?;
              writer.write_all(&length.to_be_bytes())
            },
            Message::Cancel{ index, offset, length } => {
              writer.write_all(&[0x00, 0x00, 0x00, 0x0d, 0x08])?;
              writer.write_all(&index.to_be_bytes())?;
              writer.write_all(&offset.to_be_bytes())?;
              writer.write_all(&length.to_be_bytes())
            },
            Message::Block{ index, offset, data } => {
              let len = data.len() + 9;
              let len_bytes = (len as i32).to_be_bytes();

              writer.write_all(&len_bytes)?;
              writer.write_all(&[0x07])?;
              writer.write_all(&index.to_be_bytes())?;
              writer.write_all(&offset.to_be_bytes())?;
              writer.write_all(&data)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn serialize_keepalive() {
        let mut buf = Vec::new();
        let msg = crate::Message::KeepAlive;
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf.len(), 0);
    }
    #[test]
    fn serialize_choke() {
        let mut buf = Vec::new();
        let msg = crate::Message::Choke;
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x00);
    }

    #[test]
    fn serialize_unchoke() {
        let mut buf = Vec::new();
        let msg = crate::Message::Unchoke;
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x01);
    }

    #[test]
    fn serialize_interested() {
        let mut buf = Vec::new();
        let msg = crate::Message::Interested;
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x02);
    }

    #[test]
    fn serialize_uninterested() {
        let mut buf = Vec::new();
        let msg = crate::Message::Uninterested;
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x01);
        assert_eq!(buf[4], 0x03);
    }

    #[test]
    fn serialize_have() {
        let mut buf = Vec::new();
        let msg = crate::Message::Have(23);
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
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
        let mut buf = Vec::new();
        let msg = crate::Message::Bitfield(vec![0xFF, 0xFF, 0xFF]);
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
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
        let mut buf = Vec::new();
        let msg = crate::Message::Request{ index: 666, offset: 420, length: 16384 };
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
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
        let mut buf = Vec::new();
        let msg = crate::Message::Cancel{ index: 666, offset: 420, length: 16384 };
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
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
        let mut buf = Vec::new();
        let msg = crate::Message::Block{ index: 666, offset: 420, data: vec![4, 8, 15, 16, 23, 42] };
        let res = msg.write_to(&mut buf);

        assert!(res.is_ok());
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
