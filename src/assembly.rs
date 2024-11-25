use crate::ast::ASTNode;
use std::io::{BufWriter, Result as IoResult, Write};
pub mod assembly_writer_arm64;

pub enum SupportedArchitectures {
    ARM64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegisterList {
    R0,
    R1,
    R2,
    R3,
    R4,
}


pub struct AssemblyWriter<W: Write> {
    file: BufWriter<W>,
    architecture: SupportedArchitectures,
}


trait WriteAssembly {
    fn format_register(&self, register: &RegisterList) -> String;
    fn allocate_register(&mut self) -> RegisterList;
    fn free_register(&mut self, register: RegisterList);
    fn free_all_registers(&mut self);
    fn load_register(&mut self, value: i32) -> IoResult<RegisterList>;
    fn print_register(&mut self, register: RegisterList) -> IoResult<RegisterList>;
    fn add_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList>;
    fn subtract_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList>;
    fn multiply_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList>;
    fn divide_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList>;
    fn generate_assembly_from_ast(&mut self, node: &ASTNode) -> IoResult<RegisterList>;
}
