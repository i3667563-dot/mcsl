//! Abstract Syntax Tree definitions for MCSL

/// Special argument types (@ and % prefixes)
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialArg {
    // Entity selectors (@ prefix)
    EntitySelector(String), // @a, @p, @e, @s, @r
    // Relative coordinates (@~ prefix)
    RelativeCoord(Option<f64>), // @~ or @~(value)
    // Local coordinates (%^ prefix)
    LocalCoord(Option<f64>), // %^ or %^(value)
}

/// Coordinate value (can be absolute, relative, or local)
#[derive(Debug, Clone)]
pub enum CoordValue {
    Absolute(f64),
    Relative(Option<f64>), // ~ or ~(offset)
    Local(Option<f64>),    // ^ or ^(offset)
}

/// A coordinate triplet [x, y, z]
#[derive(Debug, Clone)]
pub struct Coords {
    pub x: CoordValue,
    pub y: CoordValue,
    pub z: CoordValue,
}

/// Command argument (named or positional)
#[derive(Debug, Clone)]
pub enum CommandArg {
    Named(String, Expr), // name: value
    Positional(Expr),    // just value
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expr {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Expr>),
    Coords(Coords),
    SpecialArg(SpecialArg),
    SelectorArgs(Vec<(String, String)>), // [key=value, ...] for entity selectors
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Command(String, Vec<CommandArg>), // #command args...
    FunctionCall(String),             // #run function_name
    IfBlock(IfCondition, Block),      // if (...) { ... }
}

/// Condition for if blocks
#[derive(Debug, Clone)]
pub struct IfCondition {
    pub target: Expr,       // What to check (@a, @block, etc.)
    pub check_type: String, // What type of check (entity, block, score, etc.)
    pub operator: String,   // ==, !=, etc.
}

/// A block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub tag: Option<FunctionTag>, // $load, $tick, or None
    pub body: Block,
}

/// Function tags ($ prefix)
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionTag {
    Load,
    Tick,
}

/// Top-level item in the source file
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Function(FunctionDef),
    Statement(Statement),
}

/// Complete program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}

impl Program {
    pub fn new() -> Self {
        Program { items: Vec::new() }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
