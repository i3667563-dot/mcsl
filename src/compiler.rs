//! Main compiler logic - ties together lexer, parser, and code generator

use crate::codegen::CodeGenerator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lexer error: {0}")]
    Lexer(#[from] crate::lexer::LexerError),
    #[error("Parser error: {0}")]
    Parser(#[from] crate::parser::ParserError),
    #[error("Code generation error: {0}")]
    Codegen(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Compiler configuration
pub struct CompilerConfig {
    pub namespace: String,
    pub output_dir: PathBuf,
    pub description: String,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            namespace: "mcsldatapack".to_string(),
            output_dir: PathBuf::from("output"),
            description: "MCSL Compiled Datapack".to_string(),
        }
    }
}

/// Main compiler struct
pub struct Compiler {
    config: CompilerConfig,
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Compiler { config }
    }

    /// Compile a source file to a datapack
    pub fn compile(&self, source: &str) -> Result<(), CompilerError> {
        // Step 1: Lexical analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        // Step 2: Parsing
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        // Step 3: Code generation
        let mut codegen = CodeGenerator::new(&self.config.namespace);
        codegen.generate(&program).map_err(CompilerError::Codegen)?;

        // Step 4: Write output files
        self.write_datapack(&codegen)?;

        Ok(())
    }

    fn write_datapack(&self, codegen: &CodeGenerator) -> Result<(), CompilerError> {
        let output_dir = &self.config.output_dir;

        // Create directory structure
        let functions_dir = output_dir
            .join("data")
            .join(&self.config.namespace)
            .join("function");
        let ifblocks_dir = functions_dir.join("ifBlocks");
        let tags_dir = output_dir
            .join("data")
            .join("minecraft")
            .join("tags")
            .join("function");

        fs::create_dir_all(&functions_dir)?;
        fs::create_dir_all(&ifblocks_dir)?;
        fs::create_dir_all(&tags_dir)?;

        // Write function files
        for file in &codegen.functions {
            let path = functions_dir.join(&file.path);

            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, &file.content)?;
        }

        // Write tag files
        for (tag_path, tag_content) in codegen.generate_tags() {
            let full_path = tags_dir.join(&tag_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(full_path, &tag_content)?;
        }

        // Write pack.mcmeta
        let pack_meta = serde_json::json!({
            "pack": {
                "pack_format": 48,
                "description": self.config.description
            }
        });

        fs::write(
            output_dir.join("pack.mcmeta"),
            serde_json::to_string_pretty(&pack_meta)?,
        )?;

        Ok(())
    }
}
