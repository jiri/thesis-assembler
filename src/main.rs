#[macro_use]
extern crate lazy_static;
extern crate clap;

extern crate serde;
extern crate serde_json;

mod grammar;
mod compiler;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use clap::{App,Arg};

use compiler::*;

fn read_to_string<F: Read>(mut file: F) -> String {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Unable to read the file");
    buffer
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("file")
            .value_name("FILE")
            .help("Path to the source file")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("output")
            .value_name("OUTPUT")
            .short("o")
            .long("output")
            .help("Path to the output file")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("symfile")
            .value_name("FILE")
            .short("s")
            .long("symfile")
            .help("If set, path where the symfile will be outputted")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("whitelist")
            .value_name("FILE")
            .short("w")
            .long("whitelist")
            .help("If set, path to a file containing instruction whitelist")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("stdout")
            .long("stdout")
            .help("Output the resulting binary to stdout"))
        .get_matches();

    let filename = matches.value_of("file").expect("File name was not provided");

    let source = if filename == "-" {
        read_to_string(io::stdin())
    } else {
        read_to_string(File::open(filename).expect("Unable to open the file"))
    };

    let whitelist: Option<Vec<String>> = if let Some(whitelist_file) = matches.value_of("whitelist") {
        let file = File::open(whitelist_file).expect("Unable to open the whitelist");
        let contents = read_to_string(file);
        serde_json::from_str(contents.as_str()).ok()
    } else {
        None
    };

    match Compiler::compile(&source, whitelist) {
        Ok((binary, symbols)) => {
            let binary_res = if matches.is_present("stdout") {
                io::stdout().write_all(&binary)
            } else {
                let outfile = matches.value_of("output").unwrap_or("out.bin");
                let mut file = File::create(outfile).expect("Failed to create output file");
                file.write_all(&binary)
            };

            binary_res.expect("Failed to write file");

            if let Some(symfilepath) = matches.value_of("symfile") {
                let mut file = File::create(symfilepath).expect("Failed to create symfile");
                file.write_all(symbols.as_bytes()).expect("Failed to write symfile");
            }
        },
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
