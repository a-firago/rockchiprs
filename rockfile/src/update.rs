use crate::boot::RkTime;

use binrw::{
    binrw,    // #[binrw] attribute
};


/// Update FW header which can be found at the start of an update.img file
#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
#[br(magic = b"RKFW")]
pub struct RkUpdateHeader {
    //pub magic: [u8; 4],
    pub size: u16,
    pub version: u32,
    pub code: u32,

    pub release: RkTime,
    pub chip: u32,

    pub loader_offset: u32,
    pub loader_size: u32,

    pub image_offset: u32,
    pub image_size: u32,

    pub unknown1: u32,
    pub unknown2: u32,
    pub system_fstype: u32,
    pub backup_endpos: u32,

    pub reserved: [u8; 0x2D],
}

#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RkPartitionHeader {
    pub name: [u8; 32],
    pub filename: [u8; 60],
    pub nand_size: u32,
    pub pos: u32,
    pub nand_addr: u32,
    pub padded_size: u32,
    pub size: u32,
}

#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
#[br(magic = b"RKAF")]
pub struct RkFirmwareHeader {
    //pub magic: [u8; 4],
    pub size: u32,
    pub model: [u8; 0x22],
    pub id: [u8; 0x1e],
    pub manufacturer: [u8; 0x38],
    pub unknown1: u32,
    pub version: u32,
    pub num_parts: u32,
    pub parts: [RkPartitionHeader; 16],
    pub reserved: [u8; 0x74],
}

