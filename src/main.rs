mod gdsdk;
use std::{env, process};

fn main() {
    let mut args = env::args();
    args.next();
    if args.len() == 0 {
        eprintln!("please input with gds file");
        process::exit(0);
    }
    for file in args {
        match gdsdk::read_gdsii(&file) {
            Ok(lib) => {
                println!("{:#?}", lib);
                lib.write_to_gds();
            }
            Err(err) => eprintln!("parse file {} error: {}", file, err),
        }
    }
}
