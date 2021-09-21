// SPDX-FileCopyrightText: 2021 Rosa Richter
//
// SPDX-License-Identifier: MIT

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
    NotInterested
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
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::KeepAlive => vec![],
            Message::Choke => vec![0x00, 0x00, 0x00, 0x01, 0x00],
            Message::Unchoke => vec![0x00, 0x00, 0x00, 0x01, 0x01],
            Message::Interested => vec![0x00, 0x00, 0x00, 0x01, 0x02],
            Message::Uninterested => vec![0x00, 0x00, 0x00, 0x01, 0x03],
            Message::Have(x) => {
                let mut buf = vec![0x00, 0x00, 0x00, 0x05, 0x04];
                buf.extend(&x.to_be_bytes());
                buf
            }
            Message::Bitfield(b) => {
                let len = b.len() + 1;
                let mut buf = (len as i32).to_be_bytes().to_vec();
                buf.push(0x05);
                buf.extend(b);
                buf
            }
            Message::Request {
                index,
                offset,
                length,
            } => {
                let mut buf: Vec<u8> = vec![0x00, 0x00, 0x00, 0x0d, 0x06];
                buf.extend(index.to_be_bytes().to_vec());
                buf.extend(offset.to_be_bytes().to_vec());
                buf.extend(length.to_be_bytes().to_vec());
                buf
            }
            Message::Cancel {
                index,
                offset,
                length,
            } => {
                let mut buf = vec![0x00, 0x00, 0x00, 0x0d, 0x08];
                buf.extend(index.to_be_bytes());
                buf.extend(offset.to_be_bytes());
                buf.extend(length.to_be_bytes());
                buf
            }
            Message::Block {
                index,
                offset,
                data,
            } => {
                let len = data.len() + 9;
                let mut buf = (len as i32).to_be_bytes().to_vec();
                buf.push(0x07);
                buf.extend(&index.to_be_bytes());
                buf.extend(&offset.to_be_bytes());
                buf.extend(data);
                buf
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pwp::Message;

    #[test]
    fn serialize_keepalive() {
        let msg = Message::KeepAlive;
        let buf = msg.serialize();

        assert_eq!(buf.len(), 0);
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
        let msg = Message::Bitfield(vec![0xFF, 0xFF, 0xFF]);
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
            data: vec![4, 8, 15, 16, 23, 42],
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
