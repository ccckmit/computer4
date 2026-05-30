#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut start_dir = ".";
    let mut name_pattern: Option<&str> = None;
    let mut type_filter: Option<char> = None;

    let mut i = 1;
    while i <= args.args_len() {
        if let Some(arg) = args.get_str(i) {
            match arg {
                "-name" => {
                    i += 1;
                    name_pattern = args.get_str(i);
                }
                "-type" => {
                    i += 1;
                    if let Some(t) = args.get_str(i) {
                        type_filter = t.chars().next();
                    }
                }
                _ => {
                    if !arg.starts_with('-') && start_dir == "." {
                        start_dir = arg;
                    }
                }
            }
        }
        i += 1;
    }

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

    match find_file(start_dir, name_pattern, type_filter, &mut stat) {
        Ok(true) => 0,
        Ok(false) => 0,
        Err(_) => {
            eprintln!("find: {}: No such file or directory", start_dir);
            1
        }
    }
}

fn find_file(path: &str, name_pattern: Option<&str>, type_filter: Option<char>, stat: &mut Stat) -> Result<bool, ()> {
    let fd = match open(path, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => return Err(()),
    };

    if fstat(fd, stat).is_err() {
        let _ = close(fd);
        return Err(());
    }

    let matches_type = match type_filter {
        Some('f') => stat.r#type == InodeType::File,
        Some('d') => stat.r#type == InodeType::Directory,
        Some('l') => stat.r#type == InodeType::Device,
        _ => true,
    };

    let matches_name = match name_pattern {
        Some(pattern) => {
            let filename = get_filename(path);
            match_pattern(filename, pattern)
        }
        None => true,
    };

    if matches_type && matches_name {
        println!("{}", path);
    }

    let _ = close(fd);
    Ok(true)
}

fn get_filename(path: &str) -> &str {
    let bytes = path.as_bytes();
    let mut i = bytes.len();
    while i > 0 && bytes[i - 1] != b'/' {
        i -= 1;
    }
    unsafe { core::str::from_utf8_unchecked(&bytes[i..]) }
}

fn match_pattern(name: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();
    let mut i = 0;
    let mut j = 0;

    while i < name.len() && j < pattern.len() {
        match pattern.chars().nth(j) {
            Some('*') => {
                if j == pattern.len() - 1 {
                    return true;
                }
                let next_char = pattern.chars().nth(j + 1).unwrap_or('*');
                while i < name.len() && name.chars().nth(i) != Some(next_char) {
                    i += 1;
                }
                if i < name.len() {
                    i += 1;
                    j += 2;
                }
            }
            Some(c) => {
                if name.chars().nth(i) != Some(c) {
                    return false;
                }
                i += 1;
                j += 1;
            }
            None => break,
        }
    }

    i == name.len() && j == pattern.len()
}