use crate::assembly::{AssemblyWriter, RegisterList, SupportedArchitectures, WriteAssembly};
use crate::ast::ASTNode;
use crate::scan::Token;
use std::fs::File;
use std::io::{BufWriter, Result as IoResult, Write};

// ARM64-specific implementation
pub struct ARM64Writer {
    writer: AssemblyWriter,
    available_registers: Vec<RegisterList>, // Track available registers
}

impl ARM64Writer {
    pub fn new(file: BufWriter<File>, ast: ASTNode) -> Self {
        Self {
            writer: AssemblyWriter {
                file,
                architecture: SupportedArchitectures::ARM64,
                abstract_syntax_tree: ast,
            },
            available_registers: vec![
                RegisterList::R4,
                RegisterList::R3,
                RegisterList::R2,
                RegisterList::R1,
                RegisterList::R0,
            ],
        }
    }
}


impl WriteAssembly for ARM64Writer {
    // Helper method for register formatting
    fn format_register(&self, register: &RegisterList) -> String {
        match register {
            RegisterList::R0 => "x0",
            RegisterList::R1 => "x1",
            RegisterList::R2 => "x2",
            RegisterList::R3 => "x3",
            RegisterList::R4 => "x4",
        }.to_string()
    }
    fn allocate_register(&mut self) -> RegisterList {
        self.available_registers
            .pop()
            .expect("No available registers")
    }

    fn free_register(&mut self, register: RegisterList) {
        self.available_registers.push(register);
    }

    fn free_all_registers(&mut self) {
        self.available_registers = vec![
            RegisterList::R0,
            RegisterList::R1,
            RegisterList::R2,
            RegisterList::R3,
            RegisterList::R4,
        ];
    }

    fn load_register(&mut self, value: i32) -> IoResult<RegisterList> {
        let register = self.allocate_register();
        writeln!(
            self.writer.file,
            "\tmov {0}, #{1}\t// {0}={1}",
            self.format_register(&register),
            value
        )?;
        Ok(register)
    }

    fn print_register(&mut self, register: RegisterList) -> IoResult<RegisterList> {
        // ARM64-specific print implementation
        writeln!(self.writer.file, "    // Print register value")?;
        writeln!(self.writer.file, "    mov x0, #1           // stdout")?;
        writeln!(self.writer.file, "    mov x16, #4          // write syscall")?;
        writeln!(self.writer.file, "    svc #0x80")?;
        Ok(register)
    }

    fn add_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList> {
        let result_reg = self.allocate_register();
        writeln!(
            self.writer.file,
            "    add {}, {}, {}",
            self.format_register(&result_reg),
            self.format_register(&reg_1),
            self.format_register(&reg_2)
        )?;
        self.free_register(reg_1);
        self.free_register(reg_2);
        Ok(result_reg)
    }

    fn subtract_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList> {
        let result_reg = self.allocate_register();
        writeln!(
            self.writer.file,
            "    sub {}, {}, {}",
            self.format_register(&result_reg),
            self.format_register(&reg_1),
            self.format_register(&reg_2)
        )?;
        self.free_register(reg_1);
        self.free_register(reg_2);
        Ok(result_reg)
    }

    fn multiply_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList> {
        let result_reg = self.allocate_register();
        writeln!(
            self.writer.file,
            "    mul {}, {}, {}",
            self.format_register(&result_reg),
            self.format_register(&reg_1),
            self.format_register(&reg_2)
        )?;
        self.free_register(reg_1);
        self.free_register(reg_2);
        Ok(result_reg)
    }

    fn divide_registers(&mut self, reg_1: RegisterList, reg_2: RegisterList) -> IoResult<RegisterList> {
        let result_reg = self.allocate_register();
        writeln!(
            self.writer.file,
            "    udiv {}, {}, {}",
            self.format_register(&result_reg),
            self.format_register(&reg_1),
            self.format_register(&reg_2)
        )?;
        self.free_register(reg_1);
        self.free_register(reg_2);
        Ok(result_reg)
    }

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


// Example usage
impl ARM64Writer {
    fn compile_ast(&mut self, ast: &ASTNode) -> IoResult<()> {
        // Generate assembly from the root of the AST
        let result_reg = self.generate_assembly_from_ast(ast)?;

        // Optional: print final result
        self.print_register(result_reg.clone())?;

        // Free the final result register
        self.free_register(result_reg);

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    // Helper function to create a simple AST node
    fn create_int_node(value: i32) -> ASTNode {
        ASTNode {
            operation: Token::INT(value),
            left: None,
            right: None,
        }
    }

    // Helper function to create an operation node
    fn create_op_node(op: Token, left: ASTNode, right: ASTNode) -> ASTNode {
        ASTNode {
            operation: op,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    // Test for simple integer loading
    #[test]
    fn test_integer_loading() {
        let file = BufWriter::new(File::create("test_int_load.s").unwrap());
        let ast = create_int_node(42);

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let mut file_content = String::new();
        File::open("test_int_load.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #42"));
    }

    // Test for addition
    #[test]
    fn test_addition() {
        let file = BufWriter::new(File::create("test_addition.s").unwrap());
        let ast = create_op_node(
            Token::PLUS,
            create_int_node(10),
            create_int_node(20),
        );

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let mut file_content = String::new();
        File::open("test_addition.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #10"));
        assert!(file_content.contains("mov x1, #20"));
        assert!(file_content.contains("add"));
    }

    // Test for subtraction
    #[test]
    fn test_subtraction() {
        let file = BufWriter::new(File::create("test_subtraction.s").unwrap());
        let ast = create_op_node(
            Token::MINUS,
            create_int_node(30),
            create_int_node(15),
        );

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let mut file_content = String::new();
        File::open("test_subtraction.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #30"));
        assert!(file_content.contains("mov x1, #15"));
        assert!(file_content.contains("sub"));
    }

    // Test for multiplication
    #[test]
    fn test_multiplication() {
        let file = BufWriter::new(File::create("test_multiplication.s").unwrap());
        let ast = create_op_node(
            Token::ASTERISK,
            create_int_node(5),
            create_int_node(7),
        );

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let mut file_content = String::new();
        File::open("test_multiplication.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #5"));
        assert!(file_content.contains("mov x1, #7"));
        assert!(file_content.contains("mul"));
    }

    // Test for division
    #[test]
    fn test_division() {
        let file = BufWriter::new(File::create("test_division.s").unwrap());
        let ast = create_op_node(
            Token::SLASH,
            create_int_node(20),
            create_int_node(4),
        );

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let mut file_content = String::new();
        File::open("test_division.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #20"));
        assert!(file_content.contains("mov x1, #4"));
        assert!(file_content.contains("udiv"));
    }

    // Test for complex nested expression (5 + 3) * 2
    #[test]
    fn test_nested_expression() {
        let file = BufWriter::new(File::create("test_nested.s").unwrap());
        let ast = create_op_node(
            Token::ASTERISK,
            create_op_node(
                Token::PLUS,
                create_int_node(5),
                create_int_node(3),
            ),
            create_int_node(2),
        );

        let mut writer = ARM64Writer::new(file, ast.clone());
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string("test_nested.s").unwrap();
        // File::open("test_nested.s").unwrap().read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("mov x0, #5"));
        assert!(file_content.contains("mov x1, #3"));
        assert!(file_content.contains("add"));
        assert!(file_content.contains("mov x2, #2"));
        assert!(file_content.contains("mul"));
    }

    // // Cleanup after tests
    // #[test]
    // fn cleanup_test_files() {
    //     let test_files = [
    //         "test_int_load.s",
    //         "test_addition.s",
    //         "test_subtraction.s",
    //         "test_multiplication.s",
    //         "test_division.s",
    //         "test_nested.s"
    //     ];
    // 
    //     for file in test_files.iter() {
    //         if fs::metadata(file).is_ok() {
    //             fs::remove_file(file).unwrap();
    //         }
    //     }
    // }
}