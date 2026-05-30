#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    if args.args_len() < 2 {
        eprintln!("file: missing operand");
        return 1;
    }

    for i in 1..=args.args_len() {
        if let Some(file) = args.get_str(i) {
            match determine_file_type(file) {
                Ok(ftype) => println!("{}: {}", file, ftype),
                Err(e) => {
                    eprintln!("file: {}: {:?}", file, e);
                    return 1;
                }
            }
        }
    }

    0
}

fn determine_file_type(path: &str) -> Result<&'static str, Errno> {
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

    match stat.r#type {
        InodeType::Directory => Ok("directory"),
        InodeType::File => {
            if stat.size < 12 {
                return Ok("empty");
            }

            let fd = open(path, OpenFlag::READ_ONLY)?;
            let mut buf = [0u8; 16];
            let n = read(fd, &mut buf).unwrap_or(0);
            let _ = close(fd);

            if n >= 4 && buf[0] == 0x7f && buf[1] == 0x45 && buf[2] == 0x4c && buf[3] == 0x46 {
                return Ok("ELF executable");
            }
            if n >= 4 && buf[0] == 0x89 && buf[1] == 0x50 && buf[2] == 0x4e && buf[3] == 0x47 {
                return Ok("PNG image");
            }
            if n >= 2 && buf[0] == 0xff && buf[1] == 0xd8 {
                return Ok("JPEG image");
            }
            if n >= 4 && buf[0] == 0x47 && buf[1] == 0x49 && buf[2] == 0x46 && buf[3] == 0x38 {
                return Ok("GIF image");
            }
            if n >= 5 && buf[0] == 0x25 && buf[1] == 0x50 && buf[2] == 0x44 && buf[3] == 0x46 && buf[4] == 0x2d {
                return Ok("PDF document");
            }
            if n >= 2 && buf[0] == 0x1f && buf[1] == 0x8b {
                return Ok("gzip compressed");
            }

            Ok("data")
        }
        InodeType::Device => Ok("character special"),
        _ => Ok("unknown"),
    }
}