use super::super::{VfsResult, VfsError};

/// FAT32 Boot Sector
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BootSector {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    
    // FAT32 specific
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

impl BootSector {
    /// Parse a boot sector from a buffer
    pub fn parse(buf: &[u8]) -> VfsResult<Self> {
        if buf.len() < 512 {
            return Err(VfsError::IoError);
        }
        
        // Safety: We're reading from a properly sized buffer
        unsafe {
            let boot_sector = core::ptr::read_unaligned(buf.as_ptr() as *const BootSector);
            Ok(boot_sector)
        }
    }
    
    /// Check if this is a FAT32 filesystem
    pub fn is_fat32(&self) -> bool {
        // FAT32 has fat_size_16 == 0 and fat_size_32 > 0
        self.fat_size_16 == 0 && self.fat_size_32 > 0
    }
    
    /// Get the size of the FAT in sectors
    pub fn fat_size(&self) -> u32 {
        if self.fat_size_16 != 0 {
            self.fat_size_16 as u32
        } else {
            self.fat_size_32
        }
    }
    
    /// Get the first data sector
    pub fn first_data_sector(&self) -> u32 {
        self.reserved_sectors as u32 + (self.num_fats as u32 * self.fat_size())
    }
    
    /// Get cluster size in bytes
    pub fn cluster_size(&self) -> u32 {
        self.sectors_per_cluster as u32 * self.bytes_per_sector as u32
    }
}

/// FAT32 Directory Entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub nt_reserved: u8,
    pub create_time_tenth: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub last_access_date: u16,
    pub first_cluster_hi: u16,
    pub write_time: u16,
    pub write_date: u16,
    pub first_cluster_lo: u16,
    pub file_size: u32,
}

impl DirEntry {
    /// Check if this entry is deleted
    pub fn is_deleted(&self) -> bool {
        self.name[0] == 0xE5
    }
    
    /// Check if this is the last entry
    pub fn is_last(&self) -> bool {
        self.name[0] == 0x00
    }
    
    /// Check if this is a long filename entry
    pub fn is_lfn(&self) -> bool {
        self.attr == 0x0F
    }
    
    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.attr & 0x10 != 0
    }
    
    /// Get the first cluster number
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_hi as u32) << 16) | (self.first_cluster_lo as u32)
    }
}

// FAT32 attribute flags
pub const ATTR_READ_ONLY: u8 = 0x01;
pub const ATTR_HIDDEN: u8 = 0x02;
pub const ATTR_SYSTEM: u8 = 0x04;
pub const ATTR_VOLUME_ID: u8 = 0x08;
pub const ATTR_DIRECTORY: u8 = 0x10;
pub const ATTR_ARCHIVE: u8 = 0x20;
pub const ATTR_LONG_NAME: u8 = 0x0F;
