// Command line interface

use clap::Parser;
use std::path::PathBuf;

use crate::{compiler::CompilerError};

#[derive(Parser, Debug)]
#[command(name = "mcslc")]
#[command(author = "MCSL Project")]
#[command(version = "0.1.0")]
#[command(about = "MCSL - Minecraft Structured Language Compiler for 1.21.1", long_about = None)]
pub(crate) struct Args {
    /// Input MCSL source file
    #[arg(short = 'i', long)]
    pub(crate) input: PathBuf,

    /// Output directory for the datapack
    #[arg(short = 'o', long, default_value = "output")]
    pub(crate) output: PathBuf,

    /// Namespace for the datapack (folder name in data/)
    #[arg(short = 'n', long, default_value = "mcsldatapack")]
    pub(crate) namespace: String,

    /// Verbose output
    #[arg(short = 'v', long, default_value_t = false)]
    pub(crate) verbose: bool,
}

pub(crate) fn greeting(args: &Args) {
    println!("MCSL Compiler v0.1.0");
    println!("====================");
    println!("Input: {}", args.input.display());
    println!("Output: {}", args.output.display());
    println!("Namespace: {}", args.namespace);
    println!();
}

pub(crate) fn complete(args: &Args) {
    if args.verbose {
        println!("✓ Compilation successful!");
        println!("✓ Datapack written to: {}", args.output.display());
        println!();
        println!("To install:");
        println!(
            "  1. Copy the contents of {} to your world's datapacks folder",
            args.output.display()
        );
        println!("  2. Run /reload in Minecraft");
    } else {
        println!("Compiled successfully! Output: {}", args.output.display());
    }
}

pub(crate) fn print_error(e: CompilerError) {
    eprintln!("Compilation failed!");
    eprintln!("{}", e);

    match &e {
        CompilerError::Lexer(_le) => {
            eprintln!("\nHint: Check your syntax around the error position.");
            eprintln!("      Make sure strings are quoted and brackets are matched.");
        }
        CompilerError::Parser(_pe) => {
            eprintln!(
                "\nHint: Check that your function definitions and blocks are properly closed."
            );
        }
        CompilerError::Io(_ioe) => {
            eprintln!("\nHint: Check file permissions and disk space.");
        }
        _ => {}
    }
}
