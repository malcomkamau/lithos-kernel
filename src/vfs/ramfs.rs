use super::{VfsNode, VfsNodeRef, FileType, Permissions, VfsResult, VfsError, inode::Inode};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// In-memory file system node
pub enum RamFsNode {
    File(RamFile),
    Directory(RamDirectory),
}

/// In-memory file
pub struct RamFile {
    inode: Inode,
    data: Vec<u8>,
}

impl RamFile {
    pub fn new(inode_number: u64) -> Self {
        RamFile {
            inode: Inode::new_file(inode_number),
            data: Vec::new(),
        }
    }
}

/// In-memory directory
pub struct RamDirectory {
    inode: Inode,
    entries: BTreeMap<String, Arc<Mutex<RamFsNode>>>,
}

impl RamDirectory {
    pub fn new(inode_number: u64) -> Self {
        RamDirectory {
            inode: Inode::new_directory(inode_number),
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, node: Arc<Mutex<RamFsNode>>) {
        self.entries.insert(name, node);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<Mutex<RamFsNode>>> {
        self.entries.get(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

impl VfsNode for RamFsNode {
    fn file_type(&self) -> FileType {
        match self {
            RamFsNode::File(f) => f.inode.file_type,
            RamFsNode::Directory(d) => d.inode.file_type,
        }
    }

    fn size(&self) -> usize {
        match self {
            RamFsNode::File(f) => f.data.len(),
            RamFsNode::Directory(_) => 0,
        }
    }

    fn permissions(&self) -> Permissions {
        match self {
            RamFsNode::File(f) => f.inode.permissions,
            RamFsNode::Directory(d) => d.inode.permissions,
        }
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        match self {
            RamFsNode::File(f) => {
                if offset >= f.data.len() {
                    return Ok(0);
                }
                
                let end = core::cmp::min(offset + buf.len(), f.data.len());
                let len = end - offset;
                buf[..len].copy_from_slice(&f.data[offset..end]);
                Ok(len)
            }
            RamFsNode::Directory(_) => Err(VfsError::IsADirectory),
        }
    }

    fn write_at(&mut self, offset: usize, buf: &[u8]) -> VfsResult<usize> {
        match self {
            RamFsNode::File(f) => {
                // Extend file if necessary
                if offset + buf.len() > f.data.len() {
                    f.data.resize(offset + buf.len(), 0);
                }
                
                f.data[offset..offset + buf.len()].copy_from_slice(buf);
                f.inode.size = f.data.len();
                Ok(buf.len())
            }
            RamFsNode::Directory(_) => Err(VfsError::IsADirectory),
        }
    }

    fn readdir(&self) -> VfsResult<Vec<String>> {
        match self {
            RamFsNode::Directory(d) => Ok(d.list()),
            RamFsNode::File(_) => Err(VfsError::NotADirectory),
        }
    }

    fn lookup(&self, name: &str) -> VfsResult<VfsNodeRef> {
        match self {
            RamFsNode::Directory(d) => {
                d.get(name)
                    .ok_or(VfsError::NotFound)
                    .map(|arc| Arc::clone(arc) as VfsNodeRef)
            }
            RamFsNode::File(_) => Err(VfsError::NotADirectory),
        }
    }

    fn create(&mut self, name: &str, file_type: FileType) -> VfsResult<VfsNodeRef> {
        match self {
            RamFsNode::Directory(d) => {
                if d.entries.contains_key(name) {
                    return Err(VfsError::AlreadyExists);
                }

                let inode_number = next_inode_number();
                let node = match file_type {
                    FileType::Regular => RamFsNode::File(RamFile::new(inode_number)),
                    FileType::Directory => RamFsNode::Directory(RamDirectory::new(inode_number)),
                    _ => return Err(VfsError::IoError),
                };

                let node_ref = Arc::new(Mutex::new(node));
                d.insert(name.into(), Arc::clone(&node_ref));
                Ok(node_ref as VfsNodeRef)
            }
            RamFsNode::File(_) => Err(VfsError::NotADirectory),
        }
    }
}

/// RamFS file system
pub struct RamFs {
    root: Arc<Mutex<RamFsNode>>,
}

impl RamFs {
    pub fn new() -> Self {
        let root = Arc::new(Mutex::new(
            RamFsNode::Directory(RamDirectory::new(0))
        ));
        RamFs { root }
    }

    pub fn root_node(&self) -> VfsNodeRef {
        Arc::clone(&self.root) as VfsNodeRef
    }
}

static NEXT_INODE: AtomicU64 = AtomicU64::new(1);

fn next_inode_number() -> u64 {
    NEXT_INODE.fetch_add(1, Ordering::Relaxed)
}
