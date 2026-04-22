extern crate ovmf_prebuilt;

use std::io::{Read, Write};
use std::os::unix::fs::FileExt;

use builder::*;
#[allow(dead_code)]
enum Target {
    Json(&'static str),
    Builtin(&'static str),
}
impl Target {
    pub fn as_path(&self) -> String {
        match self {
            Target::Json(json) => json.to_string(),
            Target::Builtin(b) => b.to_string(),
        }
    }
    pub fn as_cli(&self) -> String {
        match self {
            Target::Json(json) => format!("{json}.json"),
            Target::Builtin(b) => b.to_string(),
        }
    }
}
const TARGET: Target = Target::Builtin("x86_64-unknown-uefi");

fn main() {
    cmd!(
        panic = "Failed building uefi bootloader",
        dir = "bootloader",
        "cargo build --target {} --release",
        TARGET.as_cli()
    );
    
    let bin = remove_elf_16(format!("target/{}/release/bootloader.efi", TARGET.as_path()));
    let size = get_size(&bin).unwrap();
    cmd!(
        panic = "Failed creating empty partition image",
        dir = "target",
        "qemu-img create -f raw partition.img {}B",
        size + 1024 * 1024 * 1
    );
    cmd!(
        panic = "Failed formating partition image",
        dir = "target",
        "mkfs.fat -F 32 partition.img",
    );
    cmd!(
        panic = "Failed creating partition directory to be mounted",
        dir = "target",
        "mkdir -p partition", // -p to not throw error if already exist
    );
    cmd!(
        panic = "Failed mounting FAT32 partition",
        dir = "target",
        "sudo mount partition.img partition/", // target/partition is a dir
    );
    cmd!(
        panic = "Failed creating efi dir in partition",
        dir = "target/partition",
        "sudo mkdir -p EFI/Boot",
    );
    cmd!(
        panic = "Failed copying main file in partition",
        dir = ".",
        "sudo cp {} target/partition/EFI/Boot/BootX64.efi",
        bin
    );
    cmd!(
        panic = "Failed unmounting FAT32 partition",
        dir = "target",
        "sudo umount partition.img", // target/partition is a dir
    );
    let partition_size = get_size("target/partition.img").unwrap();
    {
        let mut disk_file = std::fs::OpenOptions::new().write(true).create(true).open("target/disk.img").unwrap();
        disk_file.write(&vec![0; partition_size as usize+512]).unwrap();
    }
    cmd!(
        panic = "Failed creating MBR on disk",
        dir = "target",
        "parted disk.img mklabel msdos --script"
    );
    cmd!(
        panic = "Failed creating MBR on disk",
        dir = "target",
        "parted disk.img mkpart primary 512B {}B --script",
        partition_size + 1
    );
    let mut partition_file_buffer = Vec::with_capacity(partition_size as usize);
    std::fs::File::open("target/partition.img")
        .unwrap()
        .read_to_end(&mut partition_file_buffer)
        .unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open("target/disk.img")
        .unwrap()
        .write_at(&partition_file_buffer, 512)
        .unwrap();

    cmd!(
        dir = "target",
        "qemu-system-x86_64 -drive file=partition.img,format=raw,media=disk -bios {} -net none",
        ovmf_prebuilt::ovmf_pure_efi().display()
    );
}
