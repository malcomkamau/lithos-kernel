pub mod structs;

use super::{VfsNode, VfsNodeRef, FileType, Permissions, VfsResult, VfsError};
use crate::drivers::block::BlockDevice;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

pub use structs::*;

/// FAT32 File System
pub struct Fat32Fs {
    device: Arc<Mutex<dyn BlockDevice>>,
    boot_sector: BootSector,
}

impl Fat32Fs {
    /// Mount a FAT32 filesystem from a block device
    pub fn mount(device: Arc<Mutex<dyn BlockDevice>>) -> VfsResult<Self> {
        // Read boot sector
        let mut boot_buf = [0u8; 512];
        device.lock().read_block(0, &mut boot_buf)
            .map_err(|_| VfsError::IoError)?;
        
        let boot_sector = BootSector::parse(&boot_buf)?;
        
        // Verify it's FAT32
        if !boot_sector.is_fat32() {
            return Err(VfsError::IoError);
        }
        
        Ok(Fat32Fs {
            device,
            boot_sector,
        })
    }
    
    /// Get the root directory
    pub fn root(&self) -> VfsResult<VfsNodeRef> {
        // For now, return an error - we'll implement this next
        Err(VfsError::IoError)
    }
}
