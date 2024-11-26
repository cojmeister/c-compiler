use crate::assembly::{AssemblyWriter, RegisterList, SupportedArchitectures, WriteAssembly};
use crate::ast::ASTNode;
use std::io::{BufWriter, Result as IoResult, Write};

// ARM64-specific implementation
pub struct ARM64Writer<W: Write> {
    writer: AssemblyWriter<W>,
    available_registers: Vec<RegisterList>, // Track available registers
}

impl<W: Write> ARM64Writer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: AssemblyWriter {
                file: BufWriter::new(writer),
                architecture: SupportedArchitectures::ARM64,
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


impl<W: std::io::Write> WriteAssembly for ARM64Writer<W> {
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

    fn write_assembly_headers(&mut self) -> IoResult<()> {
        writeln!(self.writer.file, "// Auto-generated ARM64 assembly")?;
        writeln!(self.writer.file, ".arch arm64")?;
        writeln!(self.writer.file, ".text")?;
        writeln!(self.writer.file, ".global _main")?;
        writeln!(self.writer.file, ".align 2")?;
        writeln!(self.writer.file, "")?;

        // Start main function
        writeln!(self.writer.file, "_main:")?;

        Ok(())
    }

    fn write_exit_syscall(&mut self) -> IoResult<()> {
        // Standard macOS ARM64 exit syscall
        writeln!(self.writer.file, "")?;
        writeln!(self.writer.file, "    // Exit program")?;
        writeln!(self.writer.file, "    mov x0, #0           // Exit status 0")?;
        writeln!(self.writer.file, "    mov x16, #1          // Exit syscall")?;
        writeln!(self.writer.file, "    svc #0x80            // Make system call")?;

        Ok(())
    }
}


// Example usage
impl<W: std::io::Write> ARM64Writer<W> {
    pub fn compile_ast(&mut self, ast: &ASTNode) -> IoResult<()> {
        self.write_assembly_headers()?;
        
        // Generate assembly from the root of the AST
        let result_reg = self.generate_assembly_from_ast(ast)?;

        // Optional: print final result
        self.print_register(result_reg.clone())?;

        // Free the final result register
        self.free_register(result_reg);

        self.write_assembly_headers()?;

        self.writer.file.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::Token;
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Cursor};


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

    #[test]
    fn test_format_register() {
        let output = Cursor::new(Vec::<u8>::new());
        let writer = ARM64Writer::new(output);

        assert_eq!(writer.format_register(&RegisterList::R4), "x4");
        assert_eq!(writer.format_register(&RegisterList::R3), "x3");
    }


    fn setup_writer() -> (ARM64Writer<BufWriter<Cursor<Vec<u8>>>>, Cursor<Vec<u8>>) {
        let output = Cursor::new(Vec::new());
        let writer = ARM64Writer::new(BufWriter::new(output.clone()));
        (writer, output)
    }

    #[test]
    fn test_register_allocation_and_free() {
        let (mut writer, _) = setup_writer();

        // Allocate all registers
        let r0 = writer.allocate_register();
        let r1 = writer.allocate_register();
        let r2 = writer.allocate_register();

        assert_eq!(r0, RegisterList::R0);
        assert_eq!(r1, RegisterList::R1);
        assert_eq!(r2, RegisterList::R2);

        // Free a register and reallocate
        writer.free_register(r1.clone());
        let r1_reallocated = writer.allocate_register();
        assert_eq!(r1, r1_reallocated);

        // Ensure no registers are left
        writer.allocate_register(); // R3
        writer.allocate_register(); // R4
        assert!(writer.available_registers.is_empty());

        // Free all registers
        writer.free_all_registers();
        assert_eq!(writer.available_registers.len(), 5);
    }
    

    // Test for simple integer loading
    #[test]
    fn test_integer_loading() {
        let filename = "test_int_load.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_int_node(42);

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #42"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }

    // Test for addition
    #[test]
    fn test_addition() {
        let filename = "test_addition.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_op_node(
            Token::PLUS,
            create_int_node(10),
            create_int_node(20),
        );

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #10"));
        assert!(file_content.contains("mov x1, #20"));
        assert!(file_content.contains("add"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }

    // Test for subtraction
    #[test]
    fn test_subtraction() {
        let filename = "test_subtraction.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_op_node(
            Token::MINUS,
            create_int_node(30),
            create_int_node(15),
        );

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #30"));
        assert!(file_content.contains("mov x1, #15"));
        assert!(file_content.contains("sub"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }

    // Test for multiplication
    #[test]
    fn test_multiplication() {
        let filename = "test_multiplication.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_op_node(
            Token::ASTERISK,
            create_int_node(5),
            create_int_node(7),
        );

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #5"));
        assert!(file_content.contains("mov x1, #7"));
        assert!(file_content.contains("mul"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }

    // Test for division
    #[test]
    fn test_division() {
        let filename = "test_division.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_op_node(
            Token::SLASH,
            create_int_node(20),
            create_int_node(4),
        );

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #20"));
        assert!(file_content.contains("mov x1, #4"));
        assert!(file_content.contains("udiv"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }

    // Test for complex nested expression (5 + 3) * 2
    #[test]
    fn test_nested_expression() {
        let filename = "test_nested.s";
        let file = BufWriter::new(File::create(filename).unwrap());
        let ast = create_op_node(
            Token::ASTERISK,
            create_op_node(
                Token::PLUS,
                create_int_node(5),
                create_int_node(3),
            ),
            create_int_node(2),
        );

        let mut writer = ARM64Writer::new(file);
        writer.compile_ast(&ast).unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        assert!(file_content.contains("mov x0, #5"));
        assert!(file_content.contains("mov x1, #3"));
        assert!(file_content.contains("add x2, x0, x1"));
        assert!(file_content.contains("mov x1, #2"));
        assert!(file_content.contains("mul x0, x2, x1"));

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }

    }

    #[test]
    fn test_write_assembly_headers() {
        let filename = "test_headers.s";
        let file = BufWriter::new(File::create(filename).unwrap());

        let mut writer = ARM64Writer::new(file);
        writer.write_assembly_headers().unwrap();
        writer.writer.file.flush().unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        // Check for standard headers
        assert!(file_content.contains(".arch arm64"), "Missing architecture directive");
        assert!(file_content.contains(".text"), "Missing text section directive");
        assert!(file_content.contains(".global _main"), "Missing global main directive");
        assert!(file_content.contains("_main:"), "Missing main label");

        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }
    
    #[test]
    fn test_write_exit_syscall() {
        let filename = "test_exit_syscall.s";
        let file = BufWriter::new(File::create(filename).unwrap());

        let mut writer = ARM64Writer::new(file);
        writer.write_exit_syscall().unwrap();
        writer.writer.file.flush().unwrap();

        // Read generated assembly
        let file_content = fs::read_to_string(filename).unwrap();

        // Check for standard headers
        assert!(file_content.contains("    // Exit program"), "Missing exit comment");
        assert!(file_content.contains("    mov x0, #0           // Exit status 0"), "Missing exit status");
        assert!(file_content.contains("    mov x16, #1          // Exit syscall"), "Missing exit syscall");
        assert!(file_content.contains("    svc #0x80            // Make system call"), "Missing system call");


        if fs::metadata(filename).is_ok() {
            fs::remove_file(filename).unwrap();
        }
    }
}