use super::{FileType, Permissions};

/// Inode - represents file metadata
#[derive(Debug, Clone)]
pub struct Inode {
    pub file_type: FileType,
    pub size: usize,
    pub permissions: Permissions,
    pub inode_number: u64,
}

impl Inode {
    pub fn new(file_type: FileType, permissions: Permissions, inode_number: u64) -> Self {
        Inode {
            file_type,
            size: 0,
            permissions,
            inode_number,
        }
    }

    pub fn new_file(inode_number: u64) -> Self {
        Inode::new(
            FileType::Regular,
            Permissions::new(0o644),
            inode_number,
        )
    }

    pub fn new_directory(inode_number: u64) -> Self {
        Inode::new(
            FileType::Directory,
            Permissions::new(0o755),
            inode_number,
        )
    }
}
