extern crate ckb_vm as vm;

use std::env;
use std::fs::File;
use std::io::Read;
use vm::{DefaultMachine, SparseMemory};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut file = File::open(args[0].clone()).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let args2: Vec<Vec<u8>> = args.iter().map(|a| a.clone().into_bytes()).collect();

    let result = DefaultMachine::<u64, SparseMemory>::default().run(&buffer, &args2);
    println!("Result: {:?}", result);
}
