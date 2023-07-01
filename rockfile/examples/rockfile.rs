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

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Print bootloader info
    BootFile { path: PathBuf },
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
    }
}
