use std::ops::{Deref, DerefMut};

use bytes::{Bytes, BytesMut};
use util::bt::{self, InfoHash};

//----------------------------------------------------------------------------//

/// `BlockMetadata` which tracks metadata associated with a `Block` of memory.
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct BlockMetadata {
    info_hash: InfoHash,
    piece_index: u64,
    block_offset: u64,
    block_length: usize,
}

impl BlockMetadata {
    #[must_use]
    pub fn new(info_hash: InfoHash, piece_index: u64, block_offset: u64, block_length: usize) -> BlockMetadata {
        BlockMetadata {
            info_hash,
            piece_index,
            block_offset,
            block_length,
        }
    }

    #[must_use]
    pub fn with_default_hash(piece_index: u64, block_offset: u64, block_length: usize) -> BlockMetadata {
        BlockMetadata::new([0u8; bt::INFO_HASH_LEN].into(), piece_index, block_offset, block_length)
    }

    #[must_use]
    pub fn info_hash(&self) -> InfoHash {
        self.info_hash
    }

    #[must_use]
    pub fn piece_index(&self) -> u64 {
        self.piece_index
    }

    #[must_use]
    pub fn block_offset(&self) -> u64 {
        self.block_offset
    }

    #[must_use]
    pub fn block_length(&self) -> usize {
        self.block_length
    }
}

impl Default for BlockMetadata {
    fn default() -> BlockMetadata {
        BlockMetadata::new([0u8; bt::INFO_HASH_LEN].into(), 0, 0, 0)
    }
}

//----------------------------------------------------------------------------//

/// `Block` of immutable memory.
#[derive(Debug, Clone)]
pub struct Block {
    metadata: BlockMetadata,
    block_data: Bytes,
}

impl Block {
    /// Create a new `Block`.
    pub fn new(metadata: BlockMetadata, block_data: Bytes) -> Block {
        Block { metadata, block_data }
    }

    /// Access the metadata for the block.
    pub fn metadata(&self) -> BlockMetadata {
        self.metadata
    }

    pub fn into_parts(self) -> (BlockMetadata, Bytes) {
        (self.metadata, self.block_data)
    }
}

impl From<BlockMut> for Block {
    fn from(block: BlockMut) -> Block {
        Block::new(block.metadata(), block.block_data.freeze())
    }
}

impl Deref for Block {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.block_data
    }
}

//----------------------------------------------------------------------------//

/// `BlockMut` of mutable memory.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct BlockMut {
    metadata: BlockMetadata,
    block_data: BytesMut,
}

impl BlockMut {
    /// Create a new `BlockMut`.
    #[must_use]
    pub fn new(metadata: BlockMetadata, block_data: BytesMut) -> BlockMut {
        BlockMut { metadata, block_data }
    }

    /// Access the metadata for the block.
    #[must_use]
    pub fn metadata(&self) -> BlockMetadata {
        self.metadata
    }

    #[must_use]
    pub fn into_parts(self) -> (BlockMetadata, BytesMut) {
        (self.metadata, self.block_data)
    }
}

impl Deref for BlockMut {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.block_data
    }
}

impl DerefMut for BlockMut {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.block_data
    }
}
