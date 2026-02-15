use super::{VfsNode, VfsNodeRef, FileType, Permissions, VfsResult, VfsError};
use alloc::{string::String, vec::Vec, vec, sync::Arc};
use spin::Mutex;

/// Device file types
pub enum DeviceNode {
    Null,
    Zero,
    Random,
}

impl VfsNode for DeviceNode {
    fn file_type(&self) -> FileType {
        FileType::Device
    }
    
    fn size(&self) -> usize {
        0 // Device files have no size
    }
    
    fn permissions(&self) -> Permissions {
        Permissions::new(0o666) // Read/write for all
    }
    
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        match self {
            DeviceNode::Null => Ok(0), // Always EOF
            DeviceNode::Zero => {
                // Fill buffer with zeros
                for byte in buf.iter_mut() {
                    *byte = 0;
                }
                Ok(buf.len())
            }
            DeviceNode::Random => {
                // Simple pseudo-random (not cryptographically secure)
                use core::sync::atomic::{AtomicU64, Ordering};
                static SEED: AtomicU64 = AtomicU64::new(0x123456789ABCDEF0);
                
                for byte in buf.iter_mut() {
                    let seed = SEED.load(Ordering::Relaxed);
                    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    SEED.store(next, Ordering::Relaxed);
                    *byte = (next >> 16) as u8;
                }
                Ok(buf.len())
            }
        }
    }
    
    fn write_at(&mut self, _offset: usize, buf: &[u8]) -> VfsResult<usize> {
        match self {
            DeviceNode::Null => Ok(buf.len()), // Discard all writes
            DeviceNode::Zero => Err(VfsError::PermissionDenied), // Can't write to /dev/zero
            DeviceNode::Random => Err(VfsError::PermissionDenied), // Can't write to /dev/random
        }
    }
    
    fn readdir(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NotADirectory)
    }
    
    fn lookup(&self, _name: &str) -> VfsResult<VfsNodeRef> {
        Err(VfsError::NotADirectory)
    }
    
    fn create(&mut self, _name: &str, _file_type: FileType) -> VfsResult<VfsNodeRef> {
        Err(VfsError::NotADirectory)
    }
}

/// Create /dev filesystem nodes
pub fn create_dev_nodes() -> Vec<(&'static str, VfsNodeRef)> {
    vec![
        ("null", Arc::new(Mutex::new(DeviceNode::Null)) as VfsNodeRef),
        ("zero", Arc::new(Mutex::new(DeviceNode::Zero)) as VfsNodeRef),
        ("random", Arc::new(Mutex::new(DeviceNode::Random)) as VfsNodeRef),
    ]
}
