extern crate ckb_core as core;
extern crate ckb_protocol as protocol;
extern crate ckb_vm as vm;
extern crate flatbuffers;
extern crate serde_json;

mod convert;
mod syscalls;

use std::env;
use std::fs::File;
use std::io::Read;
use syscalls::MmapSyscalls;
use vm::{DefaultMachine, SparseMemory};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut file = File::open(args[0].clone()).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let args2: Vec<Vec<u8>> = args.iter().map(|a| a.clone().into_bytes()).collect();

    let mut machine = DefaultMachine::<u64, SparseMemory>::default();
    machine.add_syscall_module(Box::new(MmapSyscalls::new("data".to_string())));
    let result = machine.run(&buffer, &args2);
    println!("Result: {:?}", result);
}
