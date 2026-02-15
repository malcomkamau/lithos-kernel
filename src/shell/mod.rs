use crate::{println, vfs::ops};
use alloc::string::String;
use alloc::vec::Vec;

/// Simple shell for Lithos OS
pub struct Shell {
    cwd: String,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            cwd: String::from("/"),
        }
    }
    
    /// Execute a command
    pub fn execute(&mut self, line: &str) {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            return;
        }
        
        match parts[0] {
            "help" => self.cmd_help(),
            "ls" => self.cmd_ls(parts.get(1).copied()),
            "pwd" => self.cmd_pwd(),
            "cd" => self.cmd_cd(parts.get(1).copied()),
            "mkdir" => self.cmd_mkdir(parts.get(1).copied()),
            "touch" => self.cmd_touch(parts.get(1).copied()),
            "echo" => self.cmd_echo(&parts[1..]),
            "clear" => self.cmd_clear(),
            "" => {},
            cmd => println!("Unknown command: {}. Type 'help' for available commands.", cmd),
        }
    }
    
    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  help          - Show this help message");
        println!("  ls [path]     - List directory contents");
        println!("  pwd           - Print working directory");
        println!("  cd <path>     - Change directory");
        println!("  mkdir <path>  - Create directory");
        println!("  touch <path>  - Create empty file");
        println!("  echo <text>   - Print text");
        println!("  clear         - Clear screen");
    }
    
    fn cmd_ls(&self, path: Option<&str>) {
        let target = path.unwrap_or(&self.cwd);
        
        match ops::vfs_readdir(target) {
            Ok(entries) => {
                if entries.is_empty() {
                    println!("(empty directory)");
                } else {
                    for entry in entries {
                        println!("  {}", entry);
                    }
                }
            }
            Err(e) => println!("ls: {}", e),
        }
    }
    
    fn cmd_pwd(&self) {
        println!("{}", self.cwd);
    }
    
    fn cmd_cd(&mut self, path: Option<&str>) {
        if let Some(path) = path {
            // Simple path handling - just accept absolute paths for now
            if path.starts_with('/') {
                self.cwd = String::from(path);
                println!("Changed to {}", self.cwd);
            } else {
                println!("cd: only absolute paths supported (must start with /)");
            }
        } else {
            self.cwd = String::from("/");
            println!("Changed to /");
        }
    }
    
    fn cmd_mkdir(&self, path: Option<&str>) {
        if let Some(path) = path {
            match ops::vfs_mkdir(path) {
                Ok(_) => println!("Created directory: {}", path),
                Err(e) => println!("mkdir: {}", e),
            }
        } else {
            println!("mkdir: missing path argument");
        }
    }
    
    fn cmd_touch(&self, path: Option<&str>) {
        if let Some(path) = path {
            match ops::vfs_create(path) {
                Ok(_) => println!("Created file: {}", path),
                Err(e) => println!("touch: {}", e),
            }
        } else {
            println!("touch: missing path argument");
        }
    }
    
    fn cmd_echo(&self, args: &[&str]) {
        println!("{}", args.join(" "));
    }
    
    fn cmd_clear(&self) {
        // Simple clear - print newlines
        for _ in 0..50 {
            println!();
        }
    }
}
