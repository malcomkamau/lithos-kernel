use super::{VfsNodeRef, VfsResult, VfsError, FileType, fd_table::{FileDescriptor, OpenFlags, global_fd_table}};
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

static ROOT_FS: Mutex<Option<VfsNodeRef>> = Mutex::new(None);

/// Initialize the VFS with a root filesystem
pub fn init(root: VfsNodeRef) {
    *ROOT_FS.lock() = Some(root);
}

/// Get the root filesystem node
fn get_root() -> VfsResult<VfsNodeRef> {
    ROOT_FS.lock()
        .as_ref()
        .map(|r| r.clone())
        .ok_or(VfsError::IoError)
}

/// Resolve a path to a VFS node
pub fn resolve_path(path: &str) -> VfsResult<VfsNodeRef> {
    if path.is_empty() || !path.starts_with('/') {
        return Err(VfsError::InvalidPath);
    }

    let mut current = get_root()?;

    // Handle root path
    if path == "/" {
        return Ok(current);
    }

    // Split path and traverse
    let components: Vec<&str> = path[1..].split('/').filter(|s| !s.is_empty()).collect();

    for component in components {
        let node = current.lock().lookup(component)?;
        current = node;
    }

    Ok(current)
}

/// Open a file and return a file descriptor
pub fn vfs_open(path: &str, flags: OpenFlags) -> VfsResult<FileDescriptor> {
    let _node = resolve_path(path)?;
    
    // Allocate file descriptor
    let fd = global_fd_table().lock().alloc(flags);
    
    Ok(fd)
}

/// Read from a file descriptor
pub fn vfs_read(fd: FileDescriptor, _buf: &mut [u8]) -> VfsResult<usize> {
    let mut fd_table = global_fd_table().lock();
    let open_file = fd_table.get_mut(fd).ok_or(VfsError::NotFound)?;
    
    if !open_file.flags.read {
        return Err(VfsError::PermissionDenied);
    }

    // In a real implementation, we'd read from the actual file node
    // For now, just return 0 (EOF)
    Ok(0)
}

/// Write to a file descriptor
pub fn vfs_write(fd: FileDescriptor, buf: &[u8]) -> VfsResult<usize> {
    let mut fd_table = global_fd_table().lock();
    let open_file = fd_table.get_mut(fd).ok_or(VfsError::NotFound)?;
    
    if !open_file.flags.write {
        return Err(VfsError::PermissionDenied);
    }

    // In a real implementation, we'd write to the actual file node
    // For now, just return the buffer length
    Ok(buf.len())
}

/// Close a file descriptor
pub fn vfs_close(fd: FileDescriptor) -> VfsResult<()> {
    global_fd_table().lock().close(fd)
}

/// Create a directory
pub fn vfs_mkdir(path: &str) -> VfsResult<()> {
    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;
    
    let parent = resolve_path(parent_path)?;
    parent.lock().create(name, FileType::Directory)?;
    
    Ok(())
}

/// Create a file
pub fn vfs_create(path: &str) -> VfsResult<()> {
    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;
    
    let parent = resolve_path(parent_path)?;
    parent.lock().create(name, FileType::Regular)?;
    
    Ok(())
}

/// Read directory entries
pub fn vfs_readdir(path: &str) -> VfsResult<Vec<String>> {
    let node = resolve_path(path)?;
    let entries = node.lock().readdir()?;
    Ok(entries)
}

/// Helper function to split a path into parent and name
fn split_path(path: &str) -> VfsResult<(&str, &str)> {
    if path == "/" {
        return Err(VfsError::InvalidPath);
    }

    let last_slash = path.rfind('/').ok_or(VfsError::InvalidPath)?;
    
    let parent = if last_slash == 0 { "/" } else { &path[..last_slash] };
    let name = &path[last_slash + 1..];
    
    if name.is_empty() {
        return Err(VfsError::InvalidPath);
    }

    Ok((parent, name))
}
