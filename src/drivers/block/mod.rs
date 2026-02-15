pub mod ramdisk;

use core::fmt;

/// Standard block size (512 bytes)
pub const BLOCK_SIZE: usize = 512;

/// Block device error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    InvalidBlock,
    IoError,
    ReadOnly,
    DeviceError,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockError::InvalidBlock => write!(f, "Invalid block number"),
            BlockError::IoError => write!(f, "I/O error"),
            BlockError::ReadOnly => write!(f, "Device is read-only"),
            BlockError::DeviceError => write!(f, "Device error"),
        }
    }
}

pub type BlockResult<T> = Result<T, BlockError>;

/// Block device trait - represents block-based storage devices
pub trait BlockDevice: Send + Sync {
    /// Read a block from the device
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> BlockResult<()>;
    
    /// Write a block to the device
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> BlockResult<()>;
    
    /// Get total number of blocks
    fn block_count(&self) -> u64;
    
    /// Get block size in bytes (typically 512)
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    
    /// Check if device is read-only
    fn is_read_only(&self) -> bool {
        false
    }
}
