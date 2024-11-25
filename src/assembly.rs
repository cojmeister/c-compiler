use crate::ast::ASTNode;
use std::io::{BufWriter, Result as IoResult, Write};
use crate::scan::Token;

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
    fn generate_assembly_from_ast(&mut self, node: &ASTNode) -> IoResult<RegisterList> {
        match node.operation {
            Token::INT(n) => {
                Ok(self.load_register(n)?)
            }
            Token::PLUS => {
                // Recursively generate assembly for left and right subtrees
                let left_reg = self.generate_assembly_from_ast(
                    node.left.as_ref().expect("Missing left operand")
                )?;
                let right_reg = self.generate_assembly_from_ast(
                    node.right.as_ref().expect("Missing right operand")
                )?;

                // Perform addition
                Ok(self.add_registers(left_reg, right_reg)?)
            }
            Token::MINUS => {
                let left_reg = self.generate_assembly_from_ast(
                    node.left.as_ref().expect("Missing left operand")
                )?;
                let right_reg = self.generate_assembly_from_ast(
                    node.right.as_ref().expect("Missing right operand")
                )?;
                self.subtract_registers(left_reg, right_reg)
            }
            Token::ASTERISK => {
                let left_reg = self.generate_assembly_from_ast(
                    node.left.as_ref().expect("Missing left operand")
                )?;
                let right_reg = self.generate_assembly_from_ast(
                    node.right.as_ref().expect("Missing right operand")
                )?;
                self.multiply_registers(left_reg, right_reg)
            }
            Token::SLASH => {
                let left_reg = self.generate_assembly_from_ast(
                    node.left.as_ref().expect("Missing left operand")
                )?;
                let right_reg = self.generate_assembly_from_ast(
                    node.right.as_ref().expect("Missing right operand")
                )?;
                self.divide_registers(left_reg, right_reg)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported or invalid operation",
            )),
        }
    }
}
