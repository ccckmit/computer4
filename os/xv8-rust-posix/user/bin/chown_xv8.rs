#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("chown: missing operand");
        exit(1);
    }

    let owner = args.get_str(1).unwrap();
    let path = args.get_str(args.args_len()).unwrap();

    let (uid, gid) = parse_owner(owner);

    match chown(path, uid, gid) {
        Ok(_) => {}
        Err(e) => {
            println!("chown: cannot access '{}': {:?}", path, e);
            exit(1);
        }
    }
}

fn parse_owner(s: &str) -> (u32, u32) {
    let colon_pos = s.find(':');
    let (uid_str, gid_str) = match colon_pos {
        Some(pos) => (&s[..pos], &s[pos+1..]),
        None => (s, ""),
    };
    let uid = if uid_str.is_empty() { 0 } else { uid_str.parse().unwrap_or(0) };
    let gid = if gid_str.is_empty() { 0 } else { gid_str.parse().unwrap_or(0) };
    (uid, gid)
}