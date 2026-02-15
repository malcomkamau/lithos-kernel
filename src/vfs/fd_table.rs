use super::{VfsResult, VfsError};
use alloc::collections::BTreeMap;
use spin::Mutex;

/// File descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDescriptor(pub usize);

/// Open file handle
pub struct OpenFile {
    // Reference to VFS node would go here in a real implementation
    // For now, we'll use a simple offset tracker
    pub offset: usize,
    pub flags: OpenFlags,
}

/// File open flags
#[derive(Debug, Clone, Copy)]
pub struct OpenFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub create: bool,
}

impl OpenFlags {
    pub const fn read_only() -> Self {
        OpenFlags {
            read: true,
            write: false,
            append: false,
            create: false,
        }
    }

    pub const fn write_only() -> Self {
        OpenFlags {
            read: false,
            write: true,
            append: false,
            create: false,
        }
    }

    pub const fn read_write() -> Self {
        OpenFlags {
            read: true,
            write: true,
            append: false,
            create: false,
        }
    }
}

/// File descriptor table
pub struct FdTable {
    files: BTreeMap<FileDescriptor, OpenFile>,
    next_fd: usize,
}

impl FdTable {
    pub const fn new() -> Self {
        FdTable {
            files: BTreeMap::new(),
            next_fd: 3, // 0, 1, 2 reserved for stdin, stdout, stderr
        }
    }

    /// Allocate a new file descriptor
    pub fn alloc(&mut self, flags: OpenFlags) -> FileDescriptor {
        let fd = FileDescriptor(self.next_fd);
        self.next_fd += 1;
        
        self.files.insert(fd, OpenFile {
            offset: 0,
            flags,
        });
        
        fd
    }

    /// Get an open file by descriptor
    pub fn get(&self, fd: FileDescriptor) -> Option<&OpenFile> {
        self.files.get(&fd)
    }

    /// Get a mutable reference to an open file
    pub fn get_mut(&mut self, fd: FileDescriptor) -> Option<&mut OpenFile> {
        self.files.get_mut(&fd)
    }

    /// Close a file descriptor
    pub fn close(&mut self, fd: FileDescriptor) -> VfsResult<()> {
        self.files.remove(&fd).ok_or(VfsError::NotFound)?;
        Ok(())
    }
}

static GLOBAL_FD_TABLE: Mutex<FdTable> = Mutex::new(FdTable::new());

/// Get the global file descriptor table
pub fn global_fd_table() -> &'static Mutex<FdTable> {
    &GLOBAL_FD_TABLE
}
