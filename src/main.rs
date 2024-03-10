mod gdsdk;
use std::{env, io::Write, process};

fn main() ->std::io::Result<()>{
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
                // write gds data back
                let gds_bytes = lib.gds_bytes();
                let mut file = std::fs::File::create("new.gds")?;
                file.write(&gds_bytes)?;

            }
            Err(err) => eprintln!("parse file {} error: {}", file, err),
        }
    }
    Ok(())
}
