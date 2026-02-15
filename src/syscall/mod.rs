use crate::vfs::{fd_table::FileDescriptor, ops};
use crate::{println, print};

/// System call numbers (Linux-compatible)
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Exit = 60,
    Fork = 57,
    Exec = 59,
    Wait = 61,
}

impl Syscall {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Syscall::Read),
            1 => Some(Syscall::Write),
            2 => Some(Syscall::Open),
            3 => Some(Syscall::Close),
            60 => Some(Syscall::Exit),
            57 => Some(Syscall::Fork),
            59 => Some(Syscall::Exec),
            61 => Some(Syscall::Wait),
            _ => None,
        }
    }
}

/// System call dispatcher
/// 
/// Arguments follow x86_64 calling convention:
/// - rax: syscall number
/// - rdi, rsi, rdx, r10, r8, r9: arguments
/// - return: rax
pub fn syscall_handler(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
) -> i64 {
    let syscall = match Syscall::from_u64(syscall_num) {
        Some(s) => s,
        None => {
            println!("Unknown syscall: {}", syscall_num);
            return -1; // ENOSYS
        }
    };
    
    match syscall {
        Syscall::Read => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        Syscall::Write => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        Syscall::Open => sys_open(arg1 as *const u8, arg2 as i32),
        Syscall::Close => sys_close(arg1 as i32),
        Syscall::Exit => sys_exit(arg1 as i32),
        Syscall::Fork => sys_fork(),
        Syscall::Exec => sys_exec(arg1 as *const u8),
        Syscall::Wait => sys_wait(arg1 as *mut i32),
    }
}

/// Read from file descriptor
fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    if buf.is_null() || count == 0 {
        return -1; // EINVAL
    }
    
    // Safety: We assume the buffer is valid (in a real OS, we'd validate user pointers)
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf, count) };
    
    match ops::vfs_read(FileDescriptor(fd as usize), buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// Write to file descriptor
fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    if buf.is_null() || count == 0 {
        return -1; // EINVAL
    }
    
    // Safety: We assume the buffer is valid
    let buffer = unsafe { core::slice::from_raw_parts(buf, count) };
    
    // Special handling for stdout/stderr
    if fd == 1 || fd == 2 {
        // Write to console
        for &byte in buffer {
            print!("{}", byte as char);
        }
        return count as i64;
    }
    
    match ops::vfs_write(FileDescriptor(fd as usize), buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// Open file
fn sys_open(path: *const u8, _flags: i32) -> i64 {
    if path.is_null() {
        return -1; // EINVAL
    }
    
    // Convert C string to Rust string
    let path_str = unsafe {
        let mut len = 0;
        while *path.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(path, len);
        core::str::from_utf8(slice).unwrap_or("")
    };
    
    use crate::vfs::fd_table::OpenFlags;
    match ops::vfs_open(path_str, OpenFlags::read_write()) {
        Ok(fd) => fd.0 as i64,
        Err(_) => -1,
    }
}

/// Close file descriptor
fn sys_close(fd: i32) -> i64 {
    match ops::vfs_close(FileDescriptor(fd as usize)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Exit process
fn sys_exit(code: i32) -> i64 {
    println!("Process exited with code: {}", code);
    // In a real OS, this would terminate the current process
    // For now, we just log it
    code as i64
}

/// Fork process (not implemented yet)
fn sys_fork() -> i64 {
    println!("fork() not yet implemented");
    -1 // ENOSYS
}

/// Execute program (not implemented yet)
fn sys_exec(_path: *const u8) -> i64 {
    println!("exec() not yet implemented");
    -1 // ENOSYS
}

/// Wait for child process (not implemented yet)
fn sys_wait(_status: *mut i32) -> i64 {
    println!("wait() not yet implemented");
    -1 // ENOSYS
}
