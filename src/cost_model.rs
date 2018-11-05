use vm::instructions::{i, m, rvc};
use vm::Instruction;

pub fn instruction_cycle_costs(i: &Instruction) -> u64 {
    match i {
        Instruction::I(i) => match i {
            i::Instruction::I(i) => match i.inst() {
                i::ItypeInstruction::LD => 2,
                i::ItypeInstruction::LW => 3,
                i::ItypeInstruction::LH => 3,
                i::ItypeInstruction::LB => 3,
                i::ItypeInstruction::LWU => 3,
                i::ItypeInstruction::LHU => 3,
                i::ItypeInstruction::LBU => 3,
                _ => 1,
            },
            _ => 1,
        },
        Instruction::RVC(i) => match i {
            rvc::Instruction::Iu(i) => match i.inst() {
                rvc::ItypeUInstruction::LW => 3,
                rvc::ItypeUInstruction::LD => 2,
                _ => 1,
            },
            _ => 1,
        },
        Instruction::M(m::Instruction(i)) => match i.inst() {
            m::RtypeInstruction::MUL => 5,
            m::RtypeInstruction::MULW => 5,
            m::RtypeInstruction::MULH => 5,
            m::RtypeInstruction::MULHU => 5,
            m::RtypeInstruction::MULHSU => 5,
            m::RtypeInstruction::DIV => 16,
            m::RtypeInstruction::DIVW => 16,
            m::RtypeInstruction::DIVU => 16,
            m::RtypeInstruction::DIVUW => 16,
            m::RtypeInstruction::REM => 16,
            m::RtypeInstruction::REMW => 16,
            m::RtypeInstruction::REMU => 16,
            m::RtypeInstruction::REMUW => 16,
        },
    }
}
