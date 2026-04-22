use booter::*;

const COMPILED_BOOTLOADER_LOC: &str = "target/disk.bin";
#[cfg(not(debug_assertions))]
const PROFILE: Profile = Profile::Release;
#[cfg(debug_assertions)]
const PROFILE: Profile = Profile::Dev;

pub enum Profile {
    Release,
    Dev,
}
impl Profile {
    pub fn as_profile(self) -> &'static str {
        match self {
            Profile::Release => "release",
            Profile::Dev => "dev",
        }
    }
    pub fn as_path(self) -> &'static str {
        match self {
            Profile::Release => "release",
            Profile::Dev => "debug",
        }
    }
}

// TODO: clean build up!
fn main() {
    println!("\t[?] Building stages...");
    let mut stages = std::fs::read_dir("../bootloader")
        .unwrap()
        .map(|f| f.unwrap().file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>();
    stages.sort();
    for name in &stages {
        crate::cmd!(
            panic = format!("Failed building {}", name),
            dir = format!("../bootloader/{}", name),
            "cargo build --profile {} --target target.json",PROFILE.as_profile()
        );
    }

    println!("\t[?] Creating bootable image...");
    let raw_bootstrap = remove_elf_16(format!(
        "../bootloader/bootstrap/target/target/{}/bootstrap", PROFILE.as_path()
    ));
    ensure_size_512(&raw_bootstrap);
    std::fs::copy(raw_bootstrap, COMPILED_BOOTLOADER_LOC).unwrap();
    let mut sector_offset = 1; // 1 because bootstrap is 1 sector
    for (i, name) in stages.into_iter().filter(|name| !name.starts_with("bootstrap")).enumerate() {
        let exe = format!("../bootloader/{name}/target/target/{}/{name}", PROFILE.as_path());
        let raw = remove_elf_16(exe);
        let size = get_size(&raw);
        println!("{name} is {size} bytes ({} sectors) and at {} sectors from start", size.div_ceil(512), sector_offset);
        append_file(&COMPILED_BOOTLOADER_LOC, &raw, COMPILED_BOOTLOADER_LOC);
        setup_partition_size(
            COMPILED_BOOTLOADER_LOC.to_string(),
            size.div_ceil(512),
            sector_offset,
            i.try_into().unwrap(),
        )
        .expect("Failed opening compiled bootloader to set the partition size");
        sector_offset += size.div_ceil(512);
    }

    let size = get_size(&COMPILED_BOOTLOADER_LOC);
    println!("{COMPILED_BOOTLOADER_LOC} is {size} bytes ({} sectors)", size.div_ceil(512));
    let user_args = std::env::args()
        .skip(1)
        .map(|a| format!("{a} "))
        .collect::<String>();
    cmd!("qemu-system-x86_64 -drive file={COMPILED_BOOTLOADER_LOC},format=raw {user_args}",);
}
