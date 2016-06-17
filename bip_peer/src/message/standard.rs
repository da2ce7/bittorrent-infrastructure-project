use std::borrow::ToOwned;

use message;

use nom::{IResult, be_u32};

const BITS_PER_BYTE: u32 = 8;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct HaveMessage {
    piece_index: u32,
}

impl HaveMessage {
    pub fn new(piece_index: u32) -> HaveMessage {
        HaveMessage { piece_index: piece_index }
    }

    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], HaveMessage> {
        parse_have(bytes)
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }
}

fn parse_have(bytes: &[u8]) -> IResult<&[u8], HaveMessage> {
    map!(bytes, be_u32, |index| HaveMessage::new(index))
}

// ----------------------------------------------------------------------------//

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BitFieldMessage {
    bytes: Vec<u8>,
}

impl BitFieldMessage {
    pub fn new(total_pieces: u32) -> BitFieldMessage {
        // total_pieces is the number of bits we expect
        let bytes_needed = if total_pieces % BITS_PER_BYTE == 0 {
            total_pieces / BITS_PER_BYTE
        } else {
            (total_pieces / BITS_PER_BYTE) + 1
        };

        BitFieldMessage { bytes: vec![0u8; message::u32_to_usize(bytes_needed)] }
    }

    pub fn from_bytes(bytes: &[u8], len: u32) -> IResult<&[u8], BitFieldMessage> {
        parse_bitfield(bytes, message::u32_to_usize(len))
    }
}

fn parse_bitfield(bytes: &[u8], len: usize) -> IResult<&[u8], BitFieldMessage> {
    map!(bytes, take!(len), |b| BitFieldMessage { bytes: (b as &[u8]).to_vec() })
}

// ----------------------------------------------------------------------------//

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RequestMessage {
    piece_index: u32,
    block_offset: u32,
    block_length: usize,
}

impl RequestMessage {
    pub fn new(piece_index: u32, block_offset: u32, block_length: usize) -> RequestMessage {
        RequestMessage {
            piece_index: piece_index,
            block_offset: block_offset,
            block_length: block_length,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], RequestMessage> {
        parse_request(bytes)
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn block_offset(&self) -> u32 {
        self.block_offset
    }

    pub fn block_length(&self) -> usize {
        self.block_length
    }
}

fn parse_request(bytes: &[u8]) -> IResult<&[u8], RequestMessage> {
    map!(bytes,
         tuple!(be_u32, be_u32, be_u32),
         |(index, offset, length)| RequestMessage::new(index, offset, message::u32_to_usize(length)))
}

// ----------------------------------------------------------------------------//

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PieceMessage {
    piece_index: u32,
    block_offset: u32,
    block_length: usize,
}

impl PieceMessage {
    pub fn new(piece_index: u32, block_offset: u32, block_length: usize) -> PieceMessage {
        PieceMessage {
            piece_index: piece_index,
            block_offset: block_offset,
            block_length: block_length,
        }
    }

    pub fn from_bytes(bytes: &[u8], len: u32) -> IResult<&[u8], PieceMessage> {
        parse_piece(bytes, message::u32_to_usize(len))
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn block_offset(&self) -> u32 {
        self.block_offset
    }

    pub fn block_length(&self) -> usize {
        self.block_length
    }
}

fn parse_piece(bytes: &[u8], len: usize) -> IResult<&[u8], PieceMessage> {
    chain!(bytes,
        piece_index:  be_u32 ~
        block_offset: be_u32 ~
        block_length: value!(len) ,
        || { PieceMessage::new(piece_index, block_offset, block_length) }
    )
}

// ----------------------------------------------------------------------------//

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CancelMessage {
    piece_index: u32,
    block_offset: u32,
    block_length: usize,
}

impl CancelMessage {
    pub fn new(piece_index: u32, block_offset: u32, block_length: usize) -> CancelMessage {
        CancelMessage {
            piece_index: piece_index,
            block_offset: block_offset,
            block_length: block_length,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], CancelMessage> {
        parse_cancel(bytes)
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn block_offset(&self) -> u32 {
        self.block_offset
    }

    pub fn block_length(&self) -> usize {
        self.block_length
    }
}

fn parse_cancel(bytes: &[u8]) -> IResult<&[u8], CancelMessage> {
    map!(bytes,
         tuple!(be_u32, be_u32, be_u32),
         |(index, offset, length)| CancelMessage::new(index, offset, message::u32_to_usize(length)))
}
