//! MCSL Compiler - Minecraft Structured Language
//!
//! A compiler that translates MCSL syntax to Minecraft 1.21.1 datapacks

mod ast;
mod cli;
mod codegen;
mod compiler;
mod lexer;
mod parser;

use clap::Parser;
use std::fs;

use crate::compiler::{Compiler, CompilerConfig};

fn main() {
    let args = cli::Args::parse();

    if args.verbose {
        cli::greeting(&args);
    }

    if !args.input.exists() {
        eprintln!("Error: Input file not found: {}", args.input.display());
        std::process::exit(1);
    }

    let source = match fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Read file: {}", e);
            std::process::exit(1);
        }
    };

    let config = CompilerConfig {
        namespace: args.namespace.to_string(),
        output_dir: args.output.to_path_buf(),
        description: format!("MCSL Datapack from {}", args.input.display()),
    };

    let compiler = Compiler::new(config);
    match compiler.compile(&source) {
        Ok(()) => cli::complete(&args),
        Err(e) => cli::print_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_simple_compilation() {
        let source = r#"
            func main {
                #say "Hello World"
            }
        "#;

        let dir = tempdir().unwrap();
        let config = compiler::CompilerConfig {
            namespace: "test".to_string(),
            output_dir: dir.path().to_path_buf(),
            description: "Test".to_string(),
        };

        let compiler = compiler::Compiler::new(config);
        let result = compiler.compile(source);

        assert!(result.is_ok());

        // Check that pack.mcmeta was created
        assert!(dir.path().join("pack.mcmeta").exists());

        // Check that function was created
        assert!(dir
            .path()
            .join("data/test/functions/main.mcfunction")
            .exists());
    }

    #[test]
    fn test_tick_function() {
        let source = r#"
            $tick func game_loop {
                #say "Ticking..."
            }
        "#;

        let dir = tempdir().unwrap();
        let config = compiler::CompilerConfig {
            namespace: "test".to_string(),
            output_dir: dir.path().to_path_buf(),
            description: "Test".to_string(),
        };

        let compiler = compiler::Compiler::new(config);
        let result = compiler.compile(source);

        assert!(result.is_ok());

        // Check that tick.json was created
        assert!(dir
            .path()
            .join("data/test/functions/tags/function/tick.json")
            .exists());
    }
}
