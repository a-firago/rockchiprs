use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use binrw::io::BufReader;
use binrw::BinReaderExt;
use anyhow::{Result};

use clap::Parser;
use rockfile::boot::{
    RkBootEntry, RkBootHeader, RkBootHeaderEntry,
};
use rockfile::update::{
    RkUpdateHeader, RkFirmwareHeader, RkPartitionHeader
};

fn parse_entry(header: RkBootHeaderEntry, name: &str, reader: &mut BufReader<File>) -> Result<()> {
    for i in 0..header.count {
        reader.seek(SeekFrom::Start(
            header.offset as u64 + (header.size * i) as u64,
        ))?;

        let entry: RkBootEntry = reader.read_ne()?;

        println!("== {} Entry  {} ==", name, i);
        println!("Name: {}", String::from_utf16(entry.name.as_slice())?);
        println!("Raw: {:?}", entry);

        let mut data = Vec::new();
        data.resize(entry.data_size as usize, 0);
        reader.seek(SeekFrom::Start(entry.data_offset as u64))?;
        reader.read_exact(&mut data)?;

        let crc = crc::Crc::<u16>::new(&crc::CRC_16_IBM_3740);
        println!("Data CRC: {:x}", crc.checksum(&data));
    }

    Ok(())
}

fn parse_boot(path: &Path) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);
    let header: RkBootHeader = reader.read_ne()?;

    println!("Raw Header: {:?}", header);
    println!(
        "chip: {:?} - {}",
        header.supported_chip,
        String::from_utf8_lossy(&header.supported_chip)
    );
    parse_entry(header.entry_471, "0x471", &mut reader)?;
    parse_entry(header.entry_472, "0x472", &mut reader)?;
    parse_entry(header.entry_loader, "loader", &mut reader)?;
    Ok(())
}

fn parse_update(path: &Path) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);
    let header: RkUpdateHeader = reader.read_ne()?;

    println!("************ Update Header ************ \n {:?}", header);

    reader.seek(SeekFrom::Start(
        header.image_offset as u64,
    ))?;
    let fw_header: RkFirmwareHeader = reader.read_ne()?;

    println!("************ Firmware Header ************ \n {:?}", fw_header);
    for i in 0..fw_header.num_parts {
        println!("part = {}; size", std::str::from_utf8(&fw_header.parts[i as usize].name)?);
    }
    Ok(())
}

/// Extract chunk of a file from reader to the dst_file
fn extract_file_chunk(dst_file: &Path, reader: &mut BufReader<File>, offset :u64, size :u64) -> Result<()> {
    let prefix = dst_file.parent().unwrap();
    std::fs::create_dir_all(prefix)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dst_file)?;

    reader.seek(SeekFrom::Start(offset))?;
    let mut take = reader.take(size);
    std::io::copy(&mut take, &mut file)?;
    Ok(())
}

fn unpack_update(file_path: &Path, dst_folder: &Path) -> Result<()> {
    let mut reader = BufReader::new(File::open(file_path)?);
    let header: RkUpdateHeader = reader.read_ne()?;

    println!("Going to unpack {:?} into the {:?} folder...", file_path, dst_folder);

    reader.seek(SeekFrom::Start(
        header.loader_offset as u64,
    ))?;

    let boot_header: RkBootHeader = reader.read_ne()?;

    // println!("Update Header: {:?}", header);
    // println!("Bootloader Header: {:?}", boot_header);

    std::fs::create_dir_all(dst_folder)?;

    extract_file_chunk(&dst_folder.join("bootloader.bin"), &mut reader,
        header.loader_offset as u64, header.loader_size as u64)?;

    reader.seek(SeekFrom::Start(
        header.image_offset as u64,
    ))?;
    let fw_header: RkFirmwareHeader = reader.read_ne()?;

    // println!("************ Firmware Header ************ \n {:?}", fw_header);
    for i in 0..fw_header.num_parts {
        let part: &RkPartitionHeader = &fw_header.parts[i as usize];
        let filename = std::str::from_utf8(&part.filename)?.trim_matches(char::from(0));
        let partname = std::str::from_utf8(&part.name)?.trim_matches(char::from(0));

        println!("Unpacking partition {} into {}", partname, filename);
        extract_file_chunk(&dst_folder.join(filename),
            &mut reader, (header.image_offset + part.pos) as u64, part.size as u64)?;
    }

    Ok(())
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Print bootloader info
    BootFile {
        /// path to the bootloader file
        path: PathBuf
    },

    /// Print update.img info
    #[clap(visible_alias = "puf")]
    PrintUpdateFile {
        /// path to the update.img file
        path: PathBuf
    },

    /// Unpack update.img firmware file
     #[clap(visible_alias = "uuf")]
    UnpackUpdateFile {
        /// path to the update.img file
        file_path: PathBuf,
        /// path to the destination folder
        dst_folder: PathBuf
    },
}

#[derive(clap::Parser)]
struct Opts {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let opt = Opts::parse();

    // Commands that don't talk a device
    match opt.command {
        Command::BootFile { path } => parse_boot(&path),
        Command::PrintUpdateFile { path } => parse_update(&path),
        Command::UnpackUpdateFile { file_path, dst_folder } => unpack_update(&file_path, &dst_folder),
    }
}
