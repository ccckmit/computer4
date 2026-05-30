#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("stat: missing operand");
        exit(1);
    }

    for i in 1..=args.args_len() {
        if let Some(path) = args.get_str(i) {
            if let Err(e) = stat_file(path) {
                eprintln!("stat: cannot stat '{}': {:?}", path, e);
                exit(1);
            }
        }
    }
}

fn stat_file(path: &str) -> Result<(), Errno> {
    let fd = open(path, OpenFlag::READ_ONLY)?;
    let mut stat = Stat {
        dev: 0,
        ino: 0,
        r#type: InodeType::Free,
        nlink: 0,
        size: 0,
        mode: 0,
        uid: 0,
        gid: 0,
        blksize: 0,
        blocks: 0,
        atim_sec: 0,
        atim_nsec: 0,
        mtim_sec: 0,
        mtim_nsec: 0,
        ctim_sec: 0,
        ctim_nsec: 0,
    };
    fstat(fd, &mut stat)?;
    let _ = close(fd);

    print!("  File: {}", path);
    println!();

    let file_type = match stat.r#type {
        InodeType::Directory => "directory",
        InodeType::File => "regular file",
        InodeType::Device => "device",
        _ => "unknown",
    };

    println!("  Size: {}", stat.size);
    println!("  Filetype: {}", file_type);
    println!("  Mode: {:o}", stat.mode);
    println!("  UID: {}  GID: {}", stat.uid, stat.gid);
    println!("  Device: {}  Inode: {}", stat.dev, stat.ino);
    println!("  Links: {}", stat.nlink);
    println!("  Block size: {}", stat.blksize);
    println!("  Blocks: {}", stat.blocks);
    println!("  Access: {:>12}.{:>3}", stat.atim_sec, stat.atim_nsec);
    println!("  Modify: {:>12}.{:>3}", stat.mtim_sec, stat.mtim_nsec);
    println!("  Change: {:>12}.{:>3}", stat.ctim_sec, stat.ctim_nsec);

    Ok(())
}