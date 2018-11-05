extern crate ckb_core as core;
extern crate ckb_protocol as protocol;
extern crate clap;
extern crate flatbuffers;
extern crate serde_json;

mod convert;

use clap::{App, Arg};
use convert::convert;
use core::transaction::Transaction;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let matches = App::new("builder")
        .about("Builder utility which parses JSON and serialize to flatbuffers")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("input filename to read, stdin will be used when missing")
                .takes_value(true),
        ).arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("output filename, stdout will be used when missing")
                .takes_value(true),
        ).arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("debug mode"),
        ).get_matches();
    let mut buffer = Vec::new();
    match matches.value_of("input") {
        Some(filename) => {
            let mut file = File::open(filename).unwrap();
            file.read_to_end(&mut buffer)
        }
        None => io::stdin().read_to_end(&mut buffer),
    }.unwrap();

    if matches.occurrences_of("debug") > 0 {
        let t: Transaction = serde_json::from_slice(&buffer).unwrap();
        println!("Parsed transaction: {:?}", t);
        return;
    }

    let bytes = convert(&buffer);
    match matches.value_of("output") {
        Some(filename) => {
            let mut file = File::create(filename).unwrap();
            file.write(&bytes).unwrap();
        }
        None => {
            let stdout = io::stdout();
            stdout.lock().write(&bytes).unwrap();
        }
    }
}
