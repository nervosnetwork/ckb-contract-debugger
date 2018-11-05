// Adapted from https://github.com/NervosFoundation/ckb/tree/develop/script/src/syscalls
use convert::convert;
use std::cmp;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use vm::memory::PROT_READ;
use vm::{CoreMachine, Error as VMError, Memory, Register, Syscalls, A0, A1, A2, A3, A4, A5, A7};

pub const SUCCESS: u8 = 0;
pub const OVERRIDE_LEN: u8 = 1;

pub const MMAP_TX_SYSCALL_NUMBER: u64 = 2049;
pub const MMAP_CELL_SYSCALL_NUMBER: u64 = 2050;
pub const DEBUG_PRINT_SYSCALL_NUMBER: u64 = 2051;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Mode {
    ALL,
    PARTIAL,
}

impl Mode {
    pub fn parse_from_flag(flag: u64) -> Result<Mode, VMError> {
        match flag {
            0 => Ok(Mode::ALL),
            1 => Ok(Mode::PARTIAL),
            _ => Err(VMError::ParseError),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
enum Source {
    INPUT,
    OUTPUT,
}

impl Source {
    fn parse_from_u64(i: u64) -> Result<Source, VMError> {
        match i {
            0 => Ok(Source::INPUT),
            1 => Ok(Source::OUTPUT),
            _ => Err(VMError::ParseError),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Source::INPUT => "input",
            Source::OUTPUT => "output",
        };
        write!(f, "{}", s)
    }
}

pub struct MmapSyscalls {
    path: String,
}

impl MmapSyscalls {
    pub fn new(path: String) -> MmapSyscalls {
        MmapSyscalls { path }
    }

    fn mmap_tx<R: Register, M: Memory>(
        &mut self,
        machine: &mut CoreMachine<R, M>,
    ) -> Result<(), VMError> {
        let mut buffer = Vec::new();
        let mut file = File::open(format!("{}/tx.json", self.path))?;
        file.read_to_end(&mut buffer)?;
        let data = convert(&buffer);

        let addr = machine.registers()[A0].to_usize();
        let size_addr = machine.registers()[A1].to_usize();
        let mode = Mode::parse_from_flag(machine.registers()[A2].to_u64())?;
        let size = machine.memory_mut().load64(size_addr)? as usize;

        let (size, offset) = match mode {
            Mode::ALL => {
                if size < data.len() {
                    machine.memory_mut().store64(size_addr, data.len() as u64)?;
                    machine.registers_mut()[A0] = R::from_u8(OVERRIDE_LEN);
                } else {
                    machine.registers_mut()[A0] = R::from_u8(SUCCESS);
                }
                (data.len(), 0)
            }
            Mode::PARTIAL => {
                let offset = machine.registers()[A3].to_usize();
                let real_size = cmp::min(size, data.len() - offset);
                machine.memory_mut().store64(size_addr, real_size as u64)?;
                machine.registers_mut()[A0] = R::from_u8(SUCCESS);
                (real_size, offset)
            }
        };

        machine.memory_mut().mmap(
            addr,
            size,
            PROT_READ,
            Some(Rc::new(data.to_vec().into_boxed_slice())),
            offset,
        )?;
        Ok(())
    }

    fn mmap_cell<R: Register, M: Memory>(
        &mut self,
        machine: &mut CoreMachine<R, M>,
    ) -> Result<(), VMError> {
        let addr = machine.registers()[A0].to_usize();
        let size_addr = machine.registers()[A1].to_usize();
        let mode = Mode::parse_from_flag(machine.registers()[A2].to_u64())?;

        let index = machine.registers()[A4].to_usize();
        let source = Source::parse_from_u64(machine.registers()[A5].to_u64())?;
        let size = machine.memory_mut().load64(size_addr)? as usize;

        let mut data = Vec::new();
        let mut file = File::open(format!("{}/{}/{}.bin", self.path, source, index))?;
        file.read_to_end(&mut data)?;

        let (size, offset) = match mode {
            Mode::ALL => {
                if size < data.len() {
                    machine.memory_mut().store64(size_addr, data.len() as u64)?;
                    machine.registers_mut()[A0] = R::from_u8(OVERRIDE_LEN);
                } else {
                    machine.registers_mut()[A0] = R::from_u8(SUCCESS);
                }
                (data.len(), 0)
            }
            Mode::PARTIAL => {
                let offset = machine.registers()[A3].to_usize();
                let real_size = cmp::min(size, data.len() - offset);
                machine.memory_mut().store64(size_addr, real_size as u64)?;
                machine.registers_mut()[A0] = R::from_u8(SUCCESS);
                (real_size, offset)
            }
        };

        machine.memory_mut().mmap(
            addr,
            size,
            PROT_READ,
            Some(Rc::new(data.to_vec().into_boxed_slice())),
            offset,
        )?;
        Ok(())
    }
}

impl<R: Register, M: Memory> Syscalls<R, M> for MmapSyscalls {
    fn initialize(&mut self, _machine: &mut CoreMachine<R, M>) -> Result<(), VMError> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut CoreMachine<R, M>) -> Result<bool, VMError> {
        let number = machine.registers()[A7].to_u64();
        let processed = match number {
            MMAP_CELL_SYSCALL_NUMBER => {
                self.mmap_cell(machine)?;
                true
            }
            MMAP_TX_SYSCALL_NUMBER => {
                self.mmap_tx(machine)?;
                true
            }
            _ => false,
        };
        Ok(processed)
    }
}

pub struct DebugSyscalls {}

impl<R: Register, M: Memory> Syscalls<R, M> for DebugSyscalls {
    fn initialize(&mut self, _machine: &mut CoreMachine<R, M>) -> Result<(), VMError> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut CoreMachine<R, M>) -> Result<bool, VMError> {
        let number = machine.registers()[A7].to_u64();
        if number != DEBUG_PRINT_SYSCALL_NUMBER {
            return Ok(false);
        }

        let mut addr = machine.registers()[A0].to_usize();
        let mut buffer = Vec::new();

        loop {
            let byte = machine.memory_mut().load8(addr)?;
            if byte == 0 {
                break;
            }
            buffer.push(byte);
            addr += 1;
        }

        println!("DEBUG: {}", String::from_utf8(buffer).unwrap());
        Ok(true)
    }
}
