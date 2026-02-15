pub mod inode;
pub mod fd_table;
pub mod ramfs;
pub mod ops;
pub mod fat32;
pub mod devfs;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use core::fmt;

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Device,
    Symlink,
}

/// File permissions (Unix-style)
#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub mode: u16,
}

impl Permissions {
    pub const fn new(mode: u16) -> Self {
        Permissions { mode }
    }

    pub fn can_read(&self) -> bool {
        self.mode & 0o400 != 0
    }

    pub fn can_write(&self) -> bool {
        self.mode & 0o200 != 0
    }

    pub fn can_execute(&self) -> bool {
        self.mode & 0o100 != 0
    }
}

/// VFS error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    InvalidPath,
    IoError,
    NoSpace,
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VfsError::NotFound => write!(f, "File not found"),
            VfsError::PermissionDenied => write!(f, "Permission denied"),
            VfsError::AlreadyExists => write!(f, "File already exists"),
            VfsError::NotADirectory => write!(f, "Not a directory"),
            VfsError::IsADirectory => write!(f, "Is a directory"),
            VfsError::InvalidPath => write!(f, "Invalid path"),
            VfsError::IoError => write!(f, "I/O error"),
            VfsError::NoSpace => write!(f, "No space left"),
        }
    }
}

pub type VfsResult<T> = Result<T, VfsError>;

/// Type alias for VFS node references
pub type VfsNodeRef = Arc<Mutex<dyn VfsNode>>;

/// VFS node trait - represents files, directories, and devices
pub trait VfsNode: Send + Sync {
    /// Get the file type
    fn file_type(&self) -> FileType;
    
    /// Get file size in bytes
    fn size(&self) -> usize;
    
    /// Get file permissions
    fn permissions(&self) -> Permissions;
    
    /// Read data from file at given offset
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize>;
    
    /// Write data to file at given offset
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> VfsResult<usize>;
    
    /// List directory entries (only for directories)
    fn readdir(&self) -> VfsResult<Vec<String>>;
    
    /// Lookup a child node by name (only for directories)
    fn lookup(&self, name: &str) -> VfsResult<VfsNodeRef>;
    
    /// Create a new file in this directory
    fn create(&mut self, name: &str, file_type: FileType) -> VfsResult<VfsNodeRef>;
}
