use crate::drivers::block::{BlockDevice, BlockError, BlockResult, BLOCK_SIZE};
use x86_64::instructions::port::Port;
use spin::Mutex;

/// ATA PIO driver for IDE disks
pub struct AtaDrive {
    data_port: Mutex<Port<u16>>,
    error_port: Mutex<Port<u8>>,
    sector_count_port: Mutex<Port<u8>>,
    lba_low_port: Mutex<Port<u8>>,
    lba_mid_port: Mutex<Port<u8>>,
    lba_high_port: Mutex<Port<u8>>,
    drive_port: Mutex<Port<u8>>,
    status_port: Mutex<Port<u8>>,
    command_port: Mutex<Port<u8>>,
    is_master: bool,
}

impl AtaDrive {
    /// Create a new ATA drive (primary bus, master/slave)
    pub fn new(is_master: bool) -> Self {
        AtaDrive {
            data_port: Mutex::new(Port::new(0x1F0)),
            error_port: Mutex::new(Port::new(0x1F1)),
            sector_count_port: Mutex::new(Port::new(0x1F2)),
            lba_low_port: Mutex::new(Port::new(0x1F3)),
            lba_mid_port: Mutex::new(Port::new(0x1F4)),
            lba_high_port: Mutex::new(Port::new(0x1F5)),
            drive_port: Mutex::new(Port::new(0x1F6)),
            status_port: Mutex::new(Port::new(0x1F7)),
            command_port: Mutex::new(Port::new(0x1F7)),
            is_master,
        }
    }
    
    /// Wait for drive to be ready
    fn wait_ready(&self) {
        unsafe {
            let mut status_port = self.status_port.lock();
            loop {
                let status = status_port.read();
                if (status & 0x80) == 0 && (status & 0x08) != 0 {
                    break;
                }
            }
        }
    }
    
    /// Read a sector using PIO mode
    fn read_sector(&self, lba: u32, buf: &mut [u16; 256]) -> BlockResult<()> {
        self.wait_ready();
        
        unsafe {
            // Select drive
            let drive_select = if self.is_master { 0xE0 } else { 0xF0 };
            self.drive_port.lock().write(drive_select | ((lba >> 24) & 0x0F) as u8);
            
            // Set sector count
            self.sector_count_port.lock().write(1);
            
            // Set LBA
            self.lba_low_port.lock().write((lba & 0xFF) as u8);
            self.lba_mid_port.lock().write(((lba >> 8) & 0xFF) as u8);
            self.lba_high_port.lock().write(((lba >> 16) & 0xFF) as u8);
            
            // Send read command
            self.command_port.lock().write(0x20); // READ SECTORS
            
            self.wait_ready();
            
            // Read data
            let mut data_port = self.data_port.lock();
            for word in buf.iter_mut() {
                *word = data_port.read();
            }
        }
        
        Ok(())
    }
    
    /// Write a sector using PIO mode
    fn write_sector(&self, lba: u32, buf: &[u16; 256]) -> BlockResult<()> {
        self.wait_ready();
        
        unsafe {
            // Select drive
            let drive_select = if self.is_master { 0xE0 } else { 0xF0 };
            self.drive_port.lock().write(drive_select | ((lba >> 24) & 0x0F) as u8);
            
            // Set sector count
            self.sector_count_port.lock().write(1);
            
            // Set LBA
            self.lba_low_port.lock().write((lba & 0xFF) as u8);
            self.lba_mid_port.lock().write(((lba >> 8) & 0xFF) as u8);
            self.lba_high_port.lock().write(((lba >> 16) & 0xFF) as u8);
            
            // Send write command
            self.command_port.lock().write(0x30); // WRITE SECTORS
            
            self.wait_ready();
            
            // Write data
            let mut data_port = self.data_port.lock();
            for &word in buf.iter() {
                data_port.write(word);
            }
            
            // Flush cache
            self.command_port.lock().write(0xE7); // FLUSH CACHE
            self.wait_ready();
        }
        
        Ok(())
    }
}

impl BlockDevice for AtaDrive {
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> BlockResult<()> {
        if buf.len() < BLOCK_SIZE {
            return Err(BlockError::IoError);
        }
        
        let mut word_buf = [0u16; 256];
        self.read_sector(block_num as u32, &mut word_buf)?;
        
        // Convert u16 array to u8 array
        for (i, &word) in word_buf.iter().enumerate() {
            buf[i * 2] = (word & 0xFF) as u8;
            buf[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }
        
        Ok(())
    }
    
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> BlockResult<()> {
        if buf.len() < BLOCK_SIZE {
            return Err(BlockError::IoError);
        }
        
        let mut word_buf = [0u16; 256];
        
        // Convert u8 array to u16 array
        for i in 0..256 {
            word_buf[i] = buf[i * 2] as u16 | ((buf[i * 2 + 1] as u16) << 8);
        }
        
        self.write_sector(block_num as u32, &word_buf)?;
        
        Ok(())
    }
    
    fn block_count(&self) -> u64 {
        // For now, return a fixed size (this should be detected from drive)
        // 1GB = 2097152 sectors
        2097152
    }
}
