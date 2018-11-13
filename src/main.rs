extern crate ckb_core as core;
extern crate ckb_protocol as protocol;
extern crate ckb_vm as vm;
extern crate flatbuffers;
extern crate serde_json;

mod convert;
mod cost_model;
mod syscalls;

use cost_model::instruction_cycle_costs;
use std::env;
use std::fs::File;
use std::io::Read;
use syscalls::{DebugSyscalls, MmapSyscalls};
use vm::{CoreMachine, DefaultMachine, SparseMemory};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut file = File::open(args[0].clone()).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let args2 = normalize_arguments(args);

    let mut machine = DefaultMachine::<u64, SparseMemory>::new(Box::new(instruction_cycle_costs));
    machine.add_syscall_module(Box::new(MmapSyscalls::new("data".to_string())));
    machine.add_syscall_module(Box::new(DebugSyscalls {}));
    let result = machine.run(&buffer, &args2);
    println!("Result: {:?}", result);
    println!("Cycles: {:?}", CoreMachine::cycles(&machine));
}

fn normalize_arguments(args: Vec<String>) -> Vec<Vec<u8>> {
    args.into_iter()
        .enumerate()
        .map(|(i, arg)| {
            if i != 0 && arg.starts_with("@") {
                let filename = &arg[1..];
                let mut buffer = Vec::new();
                let mut file = File::open(filename).unwrap();
                file.read_to_end(&mut buffer).unwrap();
                buffer
            } else {
                arg.into_bytes()
            }
        }).collect()
}
