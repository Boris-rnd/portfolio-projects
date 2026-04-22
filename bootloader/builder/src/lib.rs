use std::{
    io::{self, Read, Write},
    os::unix::fs::{FileExt as _, MetadataExt},
};

pub mod command;
pub use command::*;

/// Tries to create a new file with ".bin" appended to it without all the elf that we don't support
/// Returns the path with ".bin" appended (will be usefull if we add a dynamic path in the future)
pub fn remove_elf_16(input_location: String) -> String {
    let out = format!("{input_location}.bin");
    let stat = cmd!(
        "llvm-objcopy -I elf64-x86-64 -O binary --binary-architecture=i386:x86-64 {} {}",
        input_location,
        out
    );
    if !stat.success() {
        panic!("[!] Failed to remove elf headers from bin!");
    } else {
        out
    }
}

pub fn ensure_size_512(file_name: &str) {
    let file = std::fs::File::open(file_name)
        .expect(format!("Failed to open file: {}", file_name).as_str());

    let size = file.metadata().unwrap().size();

    if size > 512 {
        panic!("[!] the size of {} is greater than 512 bytes!", file_name);
    }

    // println!("The size of {} is {}", file_name, size);
}

/// Gets the size in bytes
pub fn get_size(file_name: &str) -> u64 {
    let file = std::fs::File::open(file_name)
        .expect(format!("Failed to open file: {}", file_name).as_str());
    file.metadata().unwrap().size()
}

pub fn append_file(file1: &str, file2: &str, dest: &str) {
    let mut file_one = std::fs::OpenOptions::new()
        .read(true)
        .open(file1)
        .expect(format!("Failed to open {} for read", file1).as_str());
    let mut file_two = std::fs::OpenOptions::new()
        .read(true)
        .open(file2)
        .expect(format!("Failed to open {} for read", file2).as_str());
    let mut dest_ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(dest)
        .expect(format!("Failed to open {} for append", dest).as_str());

    let mut buffer = Vec::new();
    file_one
        .read_to_end(&mut buffer)
        .expect(format!("Failed to read to the end of the first file: {} !", file1).as_str());
    file_two
        .read_to_end(&mut buffer)
        .expect(format!("Failed to read to the end of the second file: {} !", file2).as_str());
    dest_
        .write(&buffer)
        .expect(format!("Failed to write to the destination: {} !", dest).as_str());
}

/// Partition size in sectors
pub fn setup_partition_size(
    file: String,
    partition_size: u64,
    partition_offset: u64,
    partition_idx: u8,
) -> Result<(), io::Error> {
    assert!(partition_idx < 4);
    let f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file)?;
    let mut partition = [0; 16];
    f.read_at(&mut partition, 446 + 16 * partition_idx as u64)?;
    // https://wiki.osdev.org/MBR_(x86)#Partition_table_entry_format
    //TODO Support u64
    partition[8] = partition_offset as u8;
    partition[0xC] = partition_size as u8;
    f.write_at(&partition, 446 + 16 * partition_idx as u64)?;
    Ok(())
}

/// https://users.rust-lang.org/t/concatenate-const-strings/51712/4
#[macro_export]
macro_rules! combine {
    ($A:expr, $B:expr) => {{
        const LEN: usize = $A.len() + $B.len();
        const fn combine(a: &'static str, b: &'static str) -> [u8; LEN] {
            let mut out = [0u8; LEN];
            out = copy_slice(a.as_bytes(), out, 0);
            out = copy_slice(b.as_bytes(), out, a.len());
            out
        }
        const fn copy_slice(input: &[u8], mut output: [u8; LEN], offset: usize) -> [u8; LEN] {
            let mut index = 0;
            loop {
                output[offset + index] = input[index];
                index += 1;
                if index == input.len() {
                    break;
                }
            }
            output
        }
        combine($A, $B)
    }};
}
