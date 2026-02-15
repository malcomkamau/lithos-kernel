use super::{BlockDevice, BlockError, BlockResult, BLOCK_SIZE};
use alloc::vec::Vec;
use spin::Mutex;

/// RAM disk - in-memory block device for testing
pub struct RamDisk {
    data: Mutex<Vec<u8>>,
    block_count: u64,
}

impl RamDisk {
    /// Create a new RAM disk with the specified number of blocks
    pub fn new(block_count: u64) -> Self {
        let size = (block_count as usize) * BLOCK_SIZE;
        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);
        
        RamDisk {
            data: Mutex::new(data),
            block_count,
        }
    }
    
    /// Create a RAM disk from existing data
    pub fn from_data(data: Vec<u8>) -> Self {
        let block_count = (data.len() / BLOCK_SIZE) as u64;
        
        RamDisk {
            data: Mutex::new(data),
            block_count,
        }
    }
}

impl BlockDevice for RamDisk {
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> BlockResult<()> {
        if block_num >= self.block_count {
            return Err(BlockError::InvalidBlock);
        }
        
        if buf.len() < BLOCK_SIZE {
            return Err(BlockError::IoError);
        }
        
        let data = self.data.lock();
        let offset = (block_num as usize) * BLOCK_SIZE;
        buf[..BLOCK_SIZE].copy_from_slice(&data[offset..offset + BLOCK_SIZE]);
        
        Ok(())
    }
    
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> BlockResult<()> {
        if block_num >= self.block_count {
            return Err(BlockError::InvalidBlock);
        }
        
        if buf.len() < BLOCK_SIZE {
            return Err(BlockError::IoError);
        }
        
        let mut data = self.data.lock();
        let offset = (block_num as usize) * BLOCK_SIZE;
        data[offset..offset + BLOCK_SIZE].copy_from_slice(&buf[..BLOCK_SIZE]);
        
        Ok(())
    }
    
    fn block_count(&self) -> u64 {
        self.block_count
    }
}
