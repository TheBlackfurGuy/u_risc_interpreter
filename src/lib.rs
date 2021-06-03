mod debug;
mod errors;
mod instructions;
mod mem;

use std::convert::TryInto;
use crate::instructions::{Instruction, Instruction::*};
use crate::errors::{CPUError, CPUError::*};


pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Cpu {
    reg_a: u64,
    reg_b: u64,
    reg_s: u64,
    reg_x: u64,
    cache: [u64; 65535],
    instructions: [u8; 65535],
    devices: Vec<Box<dyn Device>>
}

pub trait Device {
    fn get_address_space(&self) -> (u64, u64);
    fn load(&self, address: u64) -> u64;
    fn push(&self, address: u64, value:u64);
}

impl Cpu {
    pub fn new(instructions: [u8; 65535], devices: Vec<Box<dyn Device>>) -> Cpu {
        Cpu {
            reg_a: 0,
            reg_b: 0,
            reg_s: 0,
            reg_x: 0,
            cache: [0; 65535],
            instructions,
            devices
        }
    }

    pub fn debug(&self) -> &Cpu {
        self
    }

    pub fn tick(&mut self) -> Result<(), CPUError>{
        self.reg_x += 1;
        self.process_instruction(self.read_instruction()?)
    }
    fn read_instruction(&self) -> Result<Instruction, CPUError>{
        match self.instructions.get((self.reg_x - 1 )as usize) {
            None => Err(OutOfInstructions(format!("Out of instructions at position {}", self.reg_x))),
            Some(i) => {
                match i {
                    0 => Ok(NoOp),
                    1 => Ok(LoadBusA(self.get_args(self.reg_x)?)),
                    2 => Ok(LoadBusB(self.get_args(self.reg_x)?)),
                    3 => Ok(Add),
                    4 => Ok(Subtract),
                    5 => Ok(Multiply),
                    6 => Ok(Divide),
                    7 => Ok(CopyAB),
                    8 => Ok(CopyBA),
                    9 => Ok(SwapAB),
                    10 => Ok(PushABus(self.get_args(self.reg_x)?)),
                    11 => Ok(PushBBus(self.get_args(self.reg_x)?)),
                    12 => Ok(LoadA(self.get_args(self.reg_x)?)),
                    13 => Ok(LoadBusX(self.get_args(self.reg_x)?)),
                    14 => Ok(CopyAX),
                    15 => Ok(CopyBX),
                    16 => Ok(PushXBus(self.get_args(self.reg_x)?)),
                    17 => Ok(LoadX(self.get_args(self.reg_x)?)),
                    18 => Ok(CopyXA),
                    19 => Ok(CopyXB),
                    20 => Ok(LoadBusAS),
                    21 => Ok(LoadBusBS),
                    22 => Ok(CopyAS),
                    23 => Ok(CopyBS),
                    24 => Ok(CopyXS),
                    25 => Ok(CopySA),
                    26 => Ok(CopySB),
                    27 => Ok(CopySX),
                    28 => Ok(SwapAS),
                    29 => Ok(SwapBS),
                    30 => Ok(PushABusS),
                    31 => Ok(PushBBusS),
                    32 => Ok(LoadBusXS),
                    33 => Ok(PushXBusS),
                    34 => Ok(SkipEq),
                    35 => Ok(SkipGrEq),
                    36 => Ok(SkipGr),
                    37 => Ok(SkipLe),
                    38 => Ok(SkipLeEq),
                    e => Err(IllegalInstruction(format!("{} is not a valid instruction", e)))
                }
            }
        }
    }
    fn get_args(&self, start: u64) -> Result<u64, CPUError> {
        Ok(u64::from_be_bytes(self.instructions[(start) as usize..(start+8) as usize].try_into().unwrap()))
    }
    fn process_instruction(&mut self, inst: Instruction) -> Result<(), CPUError> {
        match inst {
            NoOp => {Ok(())}
            LoadBusA(arg) => {
                self.reg_a = self.load_base(arg)?;
                Ok(())
            }
            LoadBusB(arg) => {
                self.reg_b = self.load_base(arg)?;
                Ok(())
            }
            Add => {
                self.reg_a += self.reg_b;
                Ok(())
            }
            Subtract => {
                self.reg_a = self.reg_a - self.reg_b;
                Ok(())
            }
            Multiply => {
                self.reg_a = self.reg_a * self.reg_b;
                Ok(())
            }
            Divide => {
                self.reg_a = self.reg_a / self.reg_b;
                Ok(())
            }
            CopyAB => {
                self.reg_b = self.reg_a;
                Ok(())
            }
            CopyBA => {
                self.reg_a = self.reg_b;
                Ok(())
            }
            SwapAB => {
                std::mem::swap(&mut self.reg_a, &mut self.reg_b);
                Ok(())
            }
            PushABus(arg) => {
                self.push_base(arg, self.reg_a)?;
                Ok(())
            }
            PushBBus(_) => {}
            LoadA(_) => {}
            LoadB(_) => {}
            LoadBusX(_) => {}
            CopyAX => {}
            CopyBX => {}
            PushXBus(_) => {}
            LoadX(_) => {}
            CopyXA => {}
            CopyXB => {}
            LoadBusAS => {}
            LoadBusBS => {}
            CopyAS => {}
            CopyBS => {}
            CopyXS => {}
            CopySA => {}
            CopySB => {}
            CopySX => {}
            SwapAS => {}
            SwapBS => {}
            PushABusS => {}
            PushBBusS => {}
            LoadBusXS => {}
            PushXBusS => {}
            SkipEq => {}
            SkipGrEq => {}
            SkipGr => {}
            SkipLe => {}
            SkipLeEq => {}
        }
    }

    fn load_base(&self, arg: u64) -> Result<u64, CPUError> {
        match arg {
            0..=65535 => {
                Ok(self.cache[arg as usize])
            }
            65536..=131071 => {
                Ok(self.instructions[arg as usize] as u64)
            }
            _ => {
                let mut success;
                for device in self.devices {
                    let (min, max) = device.get_address_space();
                    if (min..=max).contains(&arg) {
                        success = Some(device.load(arg));
                    }
                }
                if success == Some{
                    Ok(success.unwrap())
                }
                else {
                    Err(IllegalAddressLoad(format!("{} is not a populated address", arg)))
                }
            }
        }
    }

    fn push_base(&mut self, arg: u64, val: u64) -> Result<(), CPUError> {
        match arg {
            0..=65535 => {
                Ok(self.cache[arg as usize] = val)
            }
            65536..=131071 => {
                Ok(self.instructions[arg as usize] = val as u8)
            }
            _ => {
                let mut success= false;
                for device in self.devices {
                    let (min, max) = device.get_address_space();
                    if (min..=max).contains(&arg) {
                        success = true;
                        device.push(arg, val)
                    }
                }
                if success {
                    Ok(())
                }
                else {
                    Err(IllegalAddressPush(format!("{} is not a populated address", arg)))
                }
            }
        }
    }
}