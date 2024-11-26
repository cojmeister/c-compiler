use crate::assembly::assembly_writer_arm64::ARM64Writer;
use crate::ast::ASTNode;
use crate::scan::scan_file;
use std::io;
use std::io::BufWriter;

mod scan;
mod ast;
mod assembly;

// Example usage
fn main() -> io::Result<()> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open("scanner_example.test")?;
    let mut reader = BufReader::new(file);

    let tokens = scan_file(&mut reader)?;


    for token in tokens.clone() {
        match token {
            Ok(token) => println!("Token: {:?}", token),
            Err(err) => println!("Error at line {}, column {}: invalid character '{}'",
                                 err.line, err.column, err.character),
        }
    }

    let node = ASTNode::parse(tokens).unwrap();
    let file = BufWriter::new(File::create("scanner_example.asm").unwrap());

    let mut writer = ARM64Writer::new(file);

    writer.compile_ast(&node).expect("Failed to write Assembly");


    Ok(())
}