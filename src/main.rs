use crate::ast::ASTNode;
use crate::scan::scan_file;
use std::io;

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

    match ASTNode::parse(tokens).unwrap().test_evaluate() {
        Ok(result) => println!("The evaluated expression gives: {:#?}", result),
        Err(err) => println!("The expression return error: {:#?}", err),
    }


    Ok(())
}