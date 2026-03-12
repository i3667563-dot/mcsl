//! MCSL Compiler - Minecraft Structured Language
//! 
//! A compiler that translates MCSL syntax to Minecraft 1.21.1 datapacks

mod ast;
mod lexer;
mod parser;
mod codegen;
mod compiler;

use clap::Parser;
use std::path::PathBuf;
use compiler::{compile_file, CompilerError};

#[derive(Parser, Debug)]
#[command(name = "mcslc")]
#[command(author = "MCSL Project")]
#[command(version = "0.1.0")]
#[command(about = "MCSL - Minecraft Structured Language Compiler for 1.21.1", long_about = None)]
struct Args {
    /// Input MCSL source file
    #[arg(short = 'i', long)]
    input: PathBuf,
    
    /// Output directory for the datapack
    #[arg(short = 'o', long, default_value = "output")]
    output: PathBuf,
    
    /// Namespace for the datapack (folder name in data/)
    #[arg(short = 'n', long, default_value = "mcsldatapack")]
    namespace: String,
    
    /// Verbose output
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    
    if args.verbose {
        println!("MCSL Compiler v0.1.0");
        println!("====================");
        println!("Input: {}", args.input.display());
        println!("Output: {}", args.output.display());
        println!("Namespace: {}", args.namespace);
        println!();
    }
    
    // Validate input file exists
    if !args.input.exists() {
        eprintln!("Error: Input file not found: {}", args.input.display());
        std::process::exit(1);
    }
    
    // Compile
    match compile_file(&args.input, &args.output, &args.namespace) {
        Ok(()) => {
            if args.verbose {
                println!("✓ Compilation successful!");
                println!("✓ Datapack written to: {}", args.output.display());
                println!();
                println!("To install:");
                println!("  1. Copy the contents of {} to your world's datapacks folder", args.output.display());
                println!("  2. Run /reload in Minecraft");
            } else {
                println!("Compiled successfully! Output: {}", args.output.display());
            }
        }
        Err(e) => {
            eprintln!("Compilation failed!");
            eprintln!("{}", e);
            
            // Provide helpful error context
            match &e {
                CompilerError::Lexer(_le) => {
                    eprintln!("\nHint: Check your syntax around the error position.");
                    eprintln!("      Make sure strings are quoted and brackets are matched.");
                }
                CompilerError::Parser(_pe) => {
                    eprintln!("\nHint: Check that your function definitions and blocks are properly closed.");
                }
                CompilerError::Io(_ioe) => {
                    eprintln!("\nHint: Check file permissions and disk space.");
                }
                _ => {}
            }
            
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        assert!(dir.path()
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
        assert!(dir.path()
            .join("data/test/functions/tags/function/tick.json")
            .exists());
    }
}
