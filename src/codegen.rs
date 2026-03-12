//! Code Generator - converts AST to Minecraft commands
//! Full support for Minecraft 1.21.1 commands

use crate::ast::*;

/// Generated function file
#[derive(Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

/// Code generator state
pub struct CodeGenerator {
    pub if_counter: u32,
    pub functions: Vec<GeneratedFile>,
    pub load_functions: Vec<String>,
    pub tick_functions: Vec<String>,
    pub namespace: String,
}

impl CodeGenerator {
    pub fn new(namespace: &str) -> Self {
        CodeGenerator {
            if_counter: 0,
            functions: Vec::new(),
            load_functions: Vec::new(),
            tick_functions: Vec::new(),
            namespace: namespace.to_string(),
        }
    }
    
    /// Generate all code from a program
    pub fn generate(&mut self, program: &Program) -> Result<(), String> {
        for item in &program.items {
            self.generate_item(item)?;
        }
        Ok(())
    }
    
    fn generate_item(&mut self, item: &TopLevelItem) -> Result<(), String> {
        match item {
            TopLevelItem::Function(func) => self.generate_function(func),
            TopLevelItem::Statement(stmt) => {
                // Top-level statements go into a default function
                let content = self.generate_statement(stmt)?;
                self.functions.push(GeneratedFile {
                    path: "main.mcfunction".to_string(),
                    content,
                });
                Ok(())
            }
        }
    }
    
    fn generate_function(&mut self, func: &FunctionDef) -> Result<(), String> {
        let mut lines = Vec::new();
        
        for stmt in &func.body.statements {
            let cmd = self.generate_statement(stmt)?;
            if !cmd.is_empty() {
                lines.push(cmd);
            }
        }
        
        let content = lines.join("\n");
        let path = format!("{}.mcfunction", func.name);
        
        self.functions.push(GeneratedFile { path, content });
        
        // Register tag functions
        if let Some(tag) = &func.tag {
            match tag {
                FunctionTag::Load => self.load_functions.push(func.name.clone()),
                FunctionTag::Tick => self.tick_functions.push(func.name.clone()),
            }
        }
        
        Ok(())
    }
    
    fn generate_statement(&mut self, stmt: &Statement) -> Result<String, String> {
        match stmt {
            Statement::Command(cmd_name, args) => {
                self.generate_command(cmd_name, args)
            }
            Statement::FunctionCall(func_name) => {
                Ok(format!("function {}:{} ", self.namespace, func_name))
            }
            Statement::IfBlock(condition, body) => {
                self.generate_if_block(condition, body)
            }
        }
    }
    
    fn generate_command(&self, cmd_name: &str, args: &[CommandArg]) -> Result<String, String> {
        let mc_cmd = self.command_name_to_vanilla(cmd_name);
        let args_str = self.format_command_args(cmd_name, args)?;
        
        if args_str.is_empty() {
            Ok(mc_cmd)
        } else {
            Ok(format!("{} {}", mc_cmd, args_str))
        }
    }
    
    fn command_name_to_vanilla(&self, name: &str) -> String {
        // Map MCSL command names to Minecraft commands
        // Support aliases for convenience
        // Full Minecraft 1.21.1 command support
        match name.to_lowercase().as_str() {
            // === CORE COMMANDS ===
            "function" | "fn" | "run" => "function",
            "execute" | "exec" => "execute",
            "data" => "data",
            "item" => "item",
            "scoreboard" | "sb" => "scoreboard",
            "cscoreboard" | "csb" => "scoreboard",
            "tag" => "tag",
            "team" => "team",
            "schedule" | "sched" => "schedule",
            "return" => "return",
            "random" | "rng" => "random",
            "tick" => "tick",
            "reload" => "reload",

            // === ENTITY & MOBILITY ===
            "summon" | "spawn" => "summon",
            "damage" | "dmg" => "damage",
            "kill" => "kill",
            "teleport" | "tp" | "tele" => "tp",
            "ride" | "mount" => "ride",
            "rotate" | "rot" => "rotate",

            // === BLOCKS & WORLD ===
            "setblock" | "setb" => "setblock",
            "fill" => "fill",
            "clone" => "clone",
            "fillbiome" | "biome" => "fillbiome",
            "place" => "place",
            "forceload" | "load" => "forceload",

            // === ITEMS & INVENTORY ===
            "give" | "g" => "give",
            "clear" | "clr" => "clear",
            "loot" => "loot",
            "use" => "use",

            // === PLAYER STATE ===
            "effect" | "eff" | "potion" => "effect",
            "enchant" | "ench" => "enchant",
            "experience" | "xp" | "exp" => "xp",
            "gamemode" | "gm" => "gamemode",
            "advancement" | "adv" | "achievement" => "advancement",
            "attribute" | "attr" => "attribute",
            "recipe" | "rec" => "recipe",
            "inputpermission" | "inputperm" => "inputpermission",

            // === DISPLAY & AUDIO ===
            "title" => "title",
            "tellraw" | "message" | "msg" => "tellraw",
            "bossbar" | "boss" => "bossbar",
            "particle" | "part" | "particles" => "particle",
            "playsound" | "sound" | "play" => "playsound",
            "stopsound" | "stops" => "stopsound",
            "jfr" => "jfr",

            // === WORLD & ENVIRONMENT ===
            "time" => "time",
            "weather" | "wx" => "weather",
            "gamerule" | "rule" | "gr" => "gamerule",
            "difficulty" | "diff" => "difficulty",
            "spawnpoint" => "spawnpoint",
            "setworldspawn" | "worldspawn" => "setworldspawn",

            // === UTILITY ===
            "say" => "say",
            "tell" | "w" | "whisper" => "tell",
            "me" => "me",
            "help" | "h" => "help",
            "datapack" | "pack" => "datapack",
            "seed" => "seed",
            "locate" | "loc" => "locate",
            "kick" => "kick",
            "list" => "list",
            "perf" => "perf",
            "save" => "save",
            "stop" => "stop",
            "whitelist" => "whitelist",
            "ban" => "ban",
            "pardon" => "pardon",
            "op" => "op",
            "deop" => "deop",

            // === DEBUG & PROFILING ===
            "debug" => "debug",
            "publish" => "publish",
            "spectate" => "spectate",

            // === STRUCTURE & WORLD GEN ===
            "jigsaw" => "jigsaw",
            "chunk" => "chunk",
            "worldborder" | "border" => "worldborder",

            // === PASS THROUGH UNKNOWN ===
            _ => name,
        }
        .to_string()
    }
    
    fn format_command_args(&self, cmd_name: &str, args: &[CommandArg]) -> Result<String, String> {
        match cmd_name.to_lowercase().as_str() {
            // === CORE COMMANDS ===
            "function" | "fn" | "run" => {
                self.format_function_args(args)
            }
            "execute" | "exec" => {
                self.format_execute_args(args)
            }
            "data" => {
                self.format_data_args(args)
            }
            "item" => {
                self.format_item_args(args)
            }
            "scoreboard" | "sb" | "cscoreboard" | "csb" => {
                self.format_scoreboard_args(args)
            }
            "tag" => {
                self.format_tag_args(args)
            }
            "team" => {
                self.format_team_args(args)
            }
            "schedule" | "sched" => {
                self.format_schedule_args(args)
            }
            "return" => {
                self.format_return_args(args)
            }
            "random" | "rng" => {
                self.format_random_args(args)
            }
            "tick" => {
                self.format_tick_args(args)
            }
            "reload" => {
                Ok("".to_string())
            }

            // === TELEPORT ===
            "tp" | "teleport" | "tele" => {
                self.format_teleport_args(args)
            }

            // === SAY / TELLRAW ===
            "say" => {
                self.format_say_args(args)
            }
            "tellraw" | "message" | "msg" => {
                self.format_tellraw_args(args)
            }
            "tell" | "w" | "whisper" => {
                self.format_tell_args(args)
            }
            "me" => {
                self.format_simple_args(args, false)
            }

            // === GIVE / CLEAR / ITEMS ===
            "give" | "g" => {
                self.format_give_args(args)
            }
            "clear" | "clr" => {
                self.format_clear_args(args)
            }
            "loot" => {
                self.format_loot_args(args)
            }
            "use" => {
                self.format_use_args(args)
            }

            // === EFFECT ===
            "effect" | "eff" | "potion" => {
                self.format_effect_args(args)
            }

            // === SUMMON ===
            "summon" | "spawn" => {
                self.format_summon_args(args)
            }

            // === SETBLOCK / FILL / CLONE ===
            "setblock" | "setb" => {
                self.format_setblock_args(args)
            }
            "fill" => {
                self.format_fill_args(args)
            }
            "clone" => {
                self.format_clone_args(args)
            }

            // === PARTICLE ===
            "particle" | "part" | "particles" => {
                self.format_particle_args(args)
            }

            // === TITLE ===
            "title" => {
                self.format_title_args(args)
            }

            // === DAMAGE ===
            "damage" | "dmg" => {
                self.format_damage_args(args)
            }

            // === ENCHANT ===
            "enchant" | "ench" => {
                self.format_enchant_args(args)
            }

            // === XP / EXPERIENCE ===
            "xp" | "experience" | "exp" => {
                self.format_xp_args(args)
            }

            // === GAMEMODE ===
            "gamemode" | "gm" => {
                self.format_gamemode_args(args)
            }

            // === ATTRIBUTE ===
            "attribute" | "attr" => {
                self.format_attribute_args(args)
            }

            // === BOSSBAR ===
            "bossbar" | "boss" => {
                self.format_bossbar_args(args)
            }

            // === PLAYSOUND / STOPSOUND ===
            "playsound" | "sound" | "play" => {
                self.format_playsound_args(args)
            }
            "stopsound" | "stops" => {
                self.format_stopsound_args(args)
            }

            // === LOCATE ===
            "locate" | "loc" => {
                self.format_locate_args(args)
            }

            // === ADVANCEMENT ===
            "advancement" | "adv" | "achievement" => {
                self.format_advancement_args(args)
            }

            // === RECIPE ===
            "recipe" | "rec" => {
                self.format_recipe_args(args)
            }

            // === WORLD BORDER ===
            "worldborder" | "border" => {
                self.format_worldborder_args(args)
            }

            // === FILL BIOME ===
            "fillbiome" | "biome" => {
                self.format_fillbiome_args(args)
            }

            // === PLACE ===
            "place" => {
                self.format_place_args(args)
            }

            // === FORCELOAD ===
            "forceload" | "load" => {
                self.format_forceload_args(args)
            }

            // === DIFFICULTY ===
            "difficulty" | "diff" => {
                self.format_difficulty_args(args)
            }

            // === GAMERULE ===
            "gamerule" | "rule" | "gr" => {
                self.format_gamerule_args(args)
            }

            // === TIME ===
            "time" => {
                self.format_time_args(args)
            }

            // === WEATHER ===
            "weather" | "wx" => {
                self.format_weather_args(args)
            }

            // === SPAWNPOINT ===
            "spawnpoint" => {
                self.format_spawnpoint_args(args)
            }

            // === SETWORLDSPAWN ===
            "setworldspawn" | "worldspawn" => {
                self.format_setworldspawn_args(args)
            }

            // === KICK ===
            "kick" => {
                self.format_kick_args(args)
            }

            // === WHITELIST ===
            "whitelist" => {
                self.format_whitelist_args(args)
            }

            // === BAN / PARDON ===
            "ban" => {
                self.format_ban_args(args)
            }
            "pardon" => {
                self.format_pardon_args(args)
            }

            // === OP / DEOP ===
            "op" => {
                self.format_op_args(args)
            }
            "deop" => {
                self.format_deop_args(args)
            }

            // === DEBUG & PROFILING ===
            "debug" => {
                self.format_debug_args(args)
            }
            "publish" => {
                self.format_publish_args(args)
            }
            "spectate" => {
                self.format_spectate_args(args)
            }
            "perf" => {
                Ok("".to_string())
            }
            "jfr" => {
                Ok("".to_string())
            }

            // === STRUCTURE & WORLD GEN ===
            "jigsaw" => {
                self.format_jigsaw_args(args)
            }
            "chunk" => {
                self.format_chunk_args(args)
            }

            // === UTILITY ===
            "help" | "h" => {
                self.format_help_args(args)
            }
            "datapack" | "pack" => {
                self.format_datapack_args(args)
            }
            "seed" => {
                Ok("".to_string())
            }
            "list" => {
                Ok("".to_string())
            }
            "save" => {
                self.format_save_args(args)
            }
            "stop" => {
                Ok("".to_string())
            }

            // === RIDE / MOUNT ===
            "ride" | "mount" => {
                self.format_ride_args(args)
            }

            // === ROTATE ===
            "rotate" | "rot" => {
                self.format_rotate_args(args)
            }

            // === INPUT PERMISSION ===
            "inputpermission" | "inputperm" => {
                self.format_inputpermission_args(args)
            }

            // === DEFAULT: Simple space-separated args ===
            _ => self.format_simple_args(args, false),
        }
    }

    // === ARGUMENT FORMATTERS ===

    fn format_teleport_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut target = "@s".to_string();
        let mut coords = "~ ~ ~".to_string();
        let mut rotation = None;
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "entity" | "target" => {
                            target = self.format_expr(value)?;
                        }
                        "coords" | "pos" | "position" | "to" => {
                            if let Expr::Coords(c) = value {
                                coords = self.format_coords(c);
                            } else if let Expr::Array(arr) = value {
                                coords = self.format_array_as_coords(arr)?;
                            }
                        }
                        "rotation" | "rot" => {
                            if let Expr::Array(arr) = value {
                                rotation = Some(self.format_array_as_coords(arr)?);
                            }
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    // Positional args: first is target, rest are coords
                    let formatted = self.format_expr(expr)?;
                    if target == "@s" {
                        target = formatted;
                    } else {
                        coords = if coords == "~ ~ ~" {
                            formatted
                        } else {
                            format!("{} {}", coords, formatted)
                        };
                    }
                }
            }
        }
        
        if let Some(rot) = rotation {
            Ok(format!("{} {} {}", target, coords, rot))
        } else {
            Ok(format!("{} {}", target, coords))
        }
    }
    
    fn format_tellraw_args(&self, args: &[CommandArg]) -> Result<String, String> {
        if args.is_empty() {
            return Ok("".to_string());
        }
        
        let mut result = Vec::new();
        
        for (i, arg) in args.iter().enumerate() {
            if i == 0 {
                // First arg is always target
                match arg {
                    CommandArg::Positional(expr) | CommandArg::Named(_, expr) => {
                        result.push(self.format_expr(expr)?);
                    }
                }
            } else {
                // Second arg is message (array or JSON string)
                match arg {
                    CommandArg::Positional(Expr::Array(items)) | CommandArg::Named(_, Expr::Array(items)) => {
                        // Format as JSON: ["text", "color", bold, italic]
                        result.push(self.format_tellraw_message(items)?);
                    }
                    CommandArg::Positional(Expr::String(s)) | CommandArg::Named(_, Expr::String(s)) => {
                        // Raw JSON string
                        result.push(s.clone());
                    }
                    CommandArg::Positional(expr) | CommandArg::Named(_, expr) => {
                        result.push(self.format_expr(expr)?);
                    }
                }
            }
        }
        
        Ok(result.join(" "))
    }
    
    fn format_tellraw_message(&self, items: &[Expr]) -> Result<String, String> {
        if items.is_empty() {
            return Ok("\"\"".to_string());
        }
        
        // Simple format: ["text", "color", bold, italic]
        let text = if let Some(Expr::String(s)) = items.get(0) {
            s.clone()
        } else {
            "".to_string()
        };
        
        let color = if let Some(Expr::String(s)) = items.get(1) {
            s.clone()
        } else {
            "white".to_string()
        };
        
        let bold = if let Some(Expr::Bool(b)) = items.get(3) {
            *b
        } else {
            false
        };
        
        let italic = if let Some(Expr::Bool(b)) = items.get(2) {
            *b
        } else {
            false
        };
        
        // Build JSON
        let mut json = format!("{{\"text\":\"{}\",\"color\":\"{}\"", text, color);
        if bold {
            json.push_str(",\"bold\":true");
        }
        if italic {
            json.push_str(",\"italic\":true");
        }
        json.push('}');
        
        Ok(json)
    }
    
    fn format_give_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut target = "@p".to_string();
        let mut item = "stone".to_string();
        let mut count = "1".to_string();
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "target" | "to" | "player" => {
                            target = self.format_expr(value)?;
                        }
                        "item" | "what" => {
                            item = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "count" | "amount" | "qty" => {
                            count = self.format_expr(value)?;
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if target == "@p" {
                        target = formatted.trim_matches('"').to_string();
                    } else if item == "stone" {
                        item = formatted.trim_matches('"').to_string();
                    } else {
                        count = formatted;
                    }
                }
            }
        }
        
        if count == "1" {
            Ok(format!("{} {}", target, item))
        } else {
            Ok(format!("{} {} {}", target, item, count))
        }
    }
    
    fn format_effect_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut parts = Vec::new();
        let mut has_subcommand = false;
        
        for arg in args {
            let formatted = match arg {
                CommandArg::Named(_, expr) => self.format_expr(expr)?,
                CommandArg::Positional(expr) => self.format_expr(expr)?,
            };
            
            // Check for subcommands
            if formatted == "give" || formatted == "clear" {
                has_subcommand = true;
            }
            
            parts.push(formatted);
        }
        
        // Auto-add "give" if no subcommand and has args
        if !has_subcommand && !parts.is_empty() {
            parts.insert(0, "give".to_string());
        }
        
        Ok(parts.join(" "))
    }
    
    fn format_summon_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut entity = "pig".to_string();
        let mut pos = "~ ~ ~".to_string();
        let mut nbt = None;
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "entity" | "what" | "type" => {
                            entity = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "pos" | "coords" | "position" | "at" => {
                            if let Expr::Coords(c) = value {
                                pos = self.format_coords(c);
                            } else if let Expr::Array(arr) = value {
                                pos = self.format_array_as_coords(arr)?;
                            }
                        }
                        "nbt" | "data" => {
                            nbt = Some(self.format_expr(value)?);
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if entity == "pig" {
                        entity = formatted.trim_matches('"').to_string();
                    } else if pos == "~ ~ ~" {
                        pos = formatted;
                    } else {
                        nbt = Some(formatted);
                    }
                }
            }
        }
        
        if let Some(nbt_data) = nbt {
            Ok(format!("{} {} {}", entity, pos, nbt_data))
        } else {
            Ok(format!("{} {}", entity, pos))
        }
    }
    
    fn format_setblock_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut pos = "~ ~ ~".to_string();
        let mut block = "stone".to_string();
        let mut mode = "replace".to_string();
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "pos" | "coords" | "position" | "at" => {
                            if let Expr::Coords(c) = value {
                                pos = self.format_coords(c);
                            } else if let Expr::Array(arr) = value {
                                pos = self.format_array_as_coords(arr)?;
                            }
                        }
                        "block" | "what" | "to" => {
                            block = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "mode" | "method" => {
                            mode = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if pos == "~ ~ ~" {
                        pos = formatted;
                    } else if block == "stone" {
                        block = formatted.trim_matches('"').to_string();
                    } else {
                        mode = formatted.trim_matches('"').to_string();
                    }
                }
            }
        }
        
        if mode != "replace" {
            Ok(format!("{} {} {}", pos, block, mode))
        } else {
            Ok(format!("{} {}", pos, block))
        }
    }
    
    fn format_fill_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut from = "~ ~ ~".to_string();
        let mut to = "~5 ~5 ~5".to_string();
        let mut block = "stone".to_string();
        let mut mode = "replace".to_string();
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "from" | "start" => {
                            if let Expr::Array(arr) = value {
                                from = self.format_array_as_coords(arr)?;
                            }
                        }
                        "to" | "end" => {
                            if let Expr::Array(arr) = value {
                                to = self.format_array_as_coords(arr)?;
                            }
                        }
                        "block" | "what" | "with" => {
                            block = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "mode" | "method" => {
                            mode = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if from == "~ ~ ~" {
                        from = formatted;
                    } else if to == "~5 ~5 ~5" {
                        to = formatted;
                    } else if block == "stone" {
                        block = formatted.trim_matches('"').to_string();
                    } else {
                        mode = formatted.trim_matches('"').to_string();
                    }
                }
            }
        }
        
        if mode != "replace" {
            Ok(format!("{} {} {} {}", from, to, block, mode))
        } else {
            Ok(format!("{} {} {}", from, to, block))
        }
    }
    
    fn format_clone_args(&self, args: &[CommandArg]) -> Result<String, String> {
        self.format_simple_args(args, false)
    }
    
    fn format_particle_args(&self, args: &[CommandArg]) -> Result<String, String> {
        self.format_simple_args(args, false)
    }
    
    fn format_title_args(&self, args: &[CommandArg]) -> Result<String, String> {
        self.format_simple_args(args, false)
    }
    
    fn format_damage_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut targets = "@s".to_string();
        let mut amount = "1.0".to_string();
        let mut damage_type = None;
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "targets" | "to" | "entity" => {
                            targets = self.format_expr(value)?;
                        }
                        "amount" | "dmg" | "damage" => {
                            amount = self.format_expr(value)?;
                        }
                        "type" | "damage_type" => {
                            damage_type = Some(self.format_expr(value)?.trim_matches('"').to_string());
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if targets == "@s" {
                        targets = formatted;
                    } else if amount == "1.0" {
                        amount = formatted;
                    } else if damage_type.is_none() {
                        damage_type = Some(formatted.trim_matches('"').to_string());
                    }
                }
            }
        }
        
        if let Some(dt) = damage_type {
            Ok(format!("{} {} {}", targets, amount, dt))
        } else {
            Ok(format!("{} {}", targets, amount))
        }
    }
    
    fn format_xp_args(&self, args: &[CommandArg]) -> Result<String, String> {
        self.format_simple_args(args, false)
    }
    
    fn format_gamemode_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut mode = "survival".to_string();
        let mut target = None;
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "mode" | "gm" | "to" => {
                            mode = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "target" | "player" | "for" => {
                            target = Some(self.format_expr(value)?);
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?.trim_matches('"').to_string();
                    if mode == "survival" {
                        mode = formatted;
                    } else if target.is_none() {
                        target = Some(formatted);
                    }
                }
            }
        }
        
        if let Some(t) = target {
            Ok(format!("{} {}", mode, t))
        } else {
            Ok(mode)
        }
    }

    fn format_spawnpoint_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // spawnpoint [targets] [<pos>] [<angle>]
        let mut target = "@a".to_string();
        let mut pos = "~ ~ ~".to_string();
        let mut angle = None;

        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "target" | "to" | "for" => {
                            target = self.format_expr(value)?;
                        }
                        "pos" | "position" | "at" => {
                            if let Expr::Coords(c) = value {
                                pos = self.format_coords(c);
                            } else if let Expr::Array(arr) = value {
                                pos = self.format_array_as_coords(arr)?;
                            }
                        }
                        "angle" | "rot" | "rotation" => {
                            angle = Some(self.format_expr(value)?);
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?;
                    if target == "@a" {
                        target = formatted;
                    } else if pos == "~ ~ ~" {
                        pos = formatted;
                    } else if angle.is_none() {
                        angle = Some(formatted);
                    }
                }
            }
        }

        if let Some(a) = angle {
            Ok(format!("{} {} {}", target, pos, a))
        } else if target == "@a" {
            Ok(pos)
        } else {
            Ok(format!("{} {}", target, pos))
        }
    }

    fn format_tell_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // tell <targets> <message>
        self.format_simple_args(args, false)
    }

    fn format_simple_args(&self, args: &[CommandArg], strip_quotes: bool) -> Result<String, String> {
        let mut result = Vec::new();
        
        for arg in args {
            let formatted = match arg {
                CommandArg::Named(name, value) => {
                    format!("{}={}", name, self.format_expr(value)?)
                }
                CommandArg::Positional(expr) => {
                    let s = self.format_expr(expr)?;
                    if strip_quotes {
                        s.trim_matches('"').to_string()
                    } else {
                        s
                    }
                }
            };
            result.push(formatted);
        }
        
        Ok(result.join(" "))
    }
    
    fn format_array_as_coords(&self, arr: &[Expr]) -> Result<String, String> {
        let mut coords = Vec::new();
        for expr in arr {
            match expr {
                Expr::Coords(c) => {
                    coords.push(self.format_coords(c));
                }
                Expr::Number(n) => {
                    coords.push(n.to_string());
                }
                Expr::String(s) => {
                    coords.push(s.clone());
                }
                _ => {}
            }
        }
        Ok(coords.join(" "))
    }
    
    fn format_args(&self, args: &[CommandArg]) -> Result<String, String> {
        let mut result = Vec::new();
        
        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    let val_str = self.format_expr(value)?;
                    result.push(format!("{}={}", name, val_str));
                }
                CommandArg::Positional(expr) => {
                    result.push(self.format_expr(expr)?);
                }
            }
        }
        
        Ok(result.join(" "))
    }
    
    fn format_expr(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::String(s) => Ok(format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))),
            Expr::Number(n) => Ok(n.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Array(items) => {
                let inner: Result<Vec<_>, _> = items.iter().map(|e| self.format_expr(e)).collect();
                Ok(format!("[{}]", inner?.join(",")))
            }
            Expr::Coords(coords) => Ok(self.format_coords(coords)),
            Expr::SpecialArg(arg) => Ok(self.format_special_arg(arg)),
            Expr::SelectorArgs(_) => Ok("".to_string()),
        }
    }
    
    fn format_coords(&self, coords: &Coords) -> String {
        let x = self.format_coord_value(&coords.x);
        let y = self.format_coord_value(&coords.y);
        let z = self.format_coord_value(&coords.z);
        format!("{} {} {}", x, y, z)
    }
    
    fn format_coord_value(&self, value: &CoordValue) -> String {
        match value {
            CoordValue::Absolute(n) => n.to_string(),
            CoordValue::Relative(Some(offset)) => format!("~{}", offset),
            CoordValue::Relative(None) => "~".to_string(),
            CoordValue::Local(Some(offset)) => format!("^{}", offset),
            CoordValue::Local(None) => "^".to_string(),
        }
    }
    
    fn format_special_arg(&self, arg: &SpecialArg) -> String {
        match arg {
            SpecialArg::EntitySelector(sel) => sel.clone(),
            SpecialArg::RelativeCoord(offset) => {
                match offset {
                    Some(o) => format!("~{}", o),
                    None => "~".to_string(),
                }
            }
            SpecialArg::LocalCoord(offset) => {
                match offset {
                    Some(o) => format!("^{}", o),
                    None => "^".to_string(),
                }
            }
        }
    }

    // === NEW ARGUMENT FORMATTERS FOR ALL COMMANDS ===

    fn format_function_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // function <name>
        self.format_simple_args(args, true)
    }

    fn format_execute_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // execute <subcommand> ... run <command>
        // For simplicity, pass through as-is
        self.format_simple_args(args, true)
    }

    fn format_data_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // data modify|get|set|merge|remove <target> ...
        self.format_simple_args(args, true)
    }

    fn format_item_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // item modify|get|set|replace <slot> ...
        self.format_simple_args(args, true)
    }

    fn format_scoreboard_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // scoreboard objectives|players|<...>
        // Support cscoreboard for custom scoreboard syntax
        self.format_simple_args(args, true)
    }

    fn format_tag_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // tag <target> add|remove|list <tag>
        self.format_simple_args(args, true)
    }

    fn format_team_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // team add|remove|modify|list <team>
        self.format_simple_args(args, true)
    }

    fn format_schedule_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // schedule function <name> <time> [replace|append]
        // schedule clear <name>
        self.format_simple_args(args, true)
    }

    fn format_return_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // return <value>
        self.format_simple_args(args, true)
    }

    fn format_random_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // random value <min> <max> [<seed>]
        // random reset [<seed>]
        // random roll <min> <max> [<seed>]
        self.format_simple_args(args, true)
    }

    fn format_tick_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // tick query|rate|sprint|step|freeze
        self.format_simple_args(args, true)
    }

    fn format_say_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // say <message>
        self.format_simple_args(args, false)
    }

    fn format_clear_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // clear [<targets>] [<item>] [<max_count>]
        let mut target = "@a".to_string();
        let mut item = "air".to_string();
        let mut max_count = "-1".to_string();

        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "target" | "from" | "player" => {
                            target = self.format_expr(value)?;
                        }
                        "item" | "what" => {
                            item = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "count" | "max" | "max_count" => {
                            max_count = self.format_expr(value)?;
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?.trim_matches('"').to_string();
                    if target == "@a" {
                        target = formatted;
                    } else if item == "air" {
                        item = formatted;
                    } else {
                        max_count = formatted;
                    }
                }
            }
        }

        if item == "air" {
            Ok(target)
        } else if max_count == "-1" {
            Ok(format!("{} {}", target, item))
        } else {
            Ok(format!("{} {} {}", target, item, max_count))
        }
    }

    fn format_loot_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // loot give|spawn|replace|insert <target> <loot_table> [<pos>|<slot>]
        self.format_simple_args(args, true)
    }

    fn format_use_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // use <item> [on <block>] [at <pos>] [facing <direction>]
        self.format_simple_args(args, true)
    }

    fn format_enchant_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // enchant <targets> <enchantment> [<level>]
        let mut targets = "@a".to_string();
        let mut enchantment = "sharpness".to_string();
        let mut level = "1".to_string();

        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "target" | "to" | "entity" => {
                            targets = self.format_expr(value)?;
                        }
                        "enchantment" | "what" | "ench" => {
                            enchantment = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "level" | "lvl" => {
                            level = self.format_expr(value)?;
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?.trim_matches('"').to_string();
                    if targets == "@a" {
                        targets = formatted;
                    } else if enchantment == "sharpness" {
                        enchantment = formatted;
                    } else {
                        level = formatted;
                    }
                }
            }
        }

        if level == "1" {
            Ok(format!("{} {}", targets, enchantment))
        } else {
            Ok(format!("{} {} {}", targets, enchantment, level))
        }
    }

    fn format_attribute_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // attribute <target> <attribute> modify|base|get ...
        self.format_simple_args(args, true)
    }

    fn format_bossbar_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // bossbar add|remove|set|get|list <id> [name] [value] [max] [color] [style] [flags]
        self.format_simple_args(args, true)
    }

    fn format_playsound_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // playsound <sound> [master|music|record|weather|block|hostile|neutral|player|ambient|voice] [<targets>] [<pos>] [<volume>] [<pitch>] [<minVolume>]
        let mut sound = "entity.player.levelup".to_string();
        let mut source = "master".to_string();
        let mut targets = "@a".to_string();
        let mut pos = "~ ~ ~".to_string();
        let mut volume = "1".to_string();
        let mut pitch = "1".to_string();
        let mut min_volume = "0".to_string();

        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "sound" | "what" => {
                            sound = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "source" | "category" => {
                            source = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "target" | "to" | "player" => {
                            targets = self.format_expr(value)?;
                        }
                        "pos" | "at" | "position" => {
                            if let Expr::Coords(c) = value {
                                pos = self.format_coords(c);
                            } else if let Expr::Array(arr) = value {
                                pos = self.format_array_as_coords(arr)?;
                            }
                        }
                        "volume" | "vol" => {
                            volume = self.format_expr(value)?;
                        }
                        "pitch" => {
                            pitch = self.format_expr(value)?;
                        }
                        "min_volume" | "min" => {
                            min_volume = self.format_expr(value)?;
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?.trim_matches('"').to_string();
                    if sound == "entity.player.levelup" {
                        sound = formatted;
                    } else if source == "master" {
                        source = formatted;
                    } else if targets == "@a" {
                        targets = formatted;
                    } else if pos == "~ ~ ~" {
                        pos = formatted;
                    } else if volume == "1" {
                        volume = formatted;
                    } else if pitch == "1" {
                        pitch = formatted;
                    } else {
                        min_volume = formatted;
                    }
                }
            }
        }

        if min_volume != "0" {
            Ok(format!("{} {} {} {} {} {} {}", sound, source, targets, pos, volume, pitch, min_volume))
        } else if pitch != "1" {
            Ok(format!("{} {} {} {} {} {}", sound, source, targets, pos, volume, pitch))
        } else if volume != "1" {
            Ok(format!("{} {} {} {} {}", sound, source, targets, pos, volume))
        } else if targets != "@a" {
            Ok(format!("{} {} {}", sound, source, targets))
        } else {
            Ok(format!("{} {}", sound, source))
        }
    }

    fn format_stopsound_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // stopsound <targets> [source] [sound]
        let mut targets = "@a".to_string();
        let mut source = "*".to_string();
        let mut sound = "*".to_string();

        for arg in args {
            match arg {
                CommandArg::Named(name, value) => {
                    match name.as_str() {
                        "target" | "to" | "player" => {
                            targets = self.format_expr(value)?;
                        }
                        "source" | "category" => {
                            source = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        "sound" | "what" => {
                            sound = self.format_expr(value)?.trim_matches('"').to_string();
                        }
                        _ => {}
                    }
                }
                CommandArg::Positional(expr) => {
                    let formatted = self.format_expr(expr)?.trim_matches('"').to_string();
                    if targets == "@a" {
                        targets = formatted;
                    } else if source == "*" {
                        source = formatted;
                    } else {
                        sound = formatted;
                    }
                }
            }
        }

        if sound != "*" {
            Ok(format!("{} {} {}", targets, source, sound))
        } else if source != "*" {
            Ok(format!("{} {}", targets, source))
        } else {
            Ok(targets)
        }
    }

    fn format_locate_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // locate structure|biome|poi <name>
        self.format_simple_args(args, true)
    }

    fn format_advancement_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // advancement grant|revoke <targets> everything|from|until|through|only <advancement>
        self.format_simple_args(args, true)
    }

    fn format_recipe_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // recipe give|take <targets> <recipe|*>
        self.format_simple_args(args, true)
    }

    fn format_worldborder_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // worldborder add|set|center|damage|warning|get
        self.format_simple_args(args, true)
    }

    fn format_fillbiome_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // fillbiome <from> <to> <biome> [filter]
        self.format_simple_args(args, true)
    }

    fn format_place_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // place feature|template|jigsaw <name> [<pos>] [<rotation>] [<mirror>] [<integration>]
        self.format_simple_args(args, true)
    }

    fn format_forceload_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // forceload add|remove|removeall|query [<from>] [<to>]
        self.format_simple_args(args, true)
    }

    fn format_difficulty_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // difficulty <level> [lock]
        self.format_simple_args(args, true)
    }

    fn format_gamerule_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // gamerule <rule> [value]
        self.format_simple_args(args, true)
    }

    fn format_time_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // time set|query|add <value>
        self.format_simple_args(args, true)
    }

    fn format_weather_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // weather clear|rain|thunder [<duration>]
        self.format_simple_args(args, true)
    }

    fn format_setworldspawn_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // setworldspawn [<pos>]
        self.format_simple_args(args, true)
    }

    fn format_kick_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // kick <targets> [reason]
        self.format_simple_args(args, false)
    }

    fn format_whitelist_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // whitelist add|remove|list|on|off|reload [<targets>]
        self.format_simple_args(args, true)
    }

    fn format_ban_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // ban <targets> [reason]
        self.format_simple_args(args, false)
    }

    fn format_pardon_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // pardon <targets>
        self.format_simple_args(args, true)
    }

    fn format_op_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // op <targets>
        self.format_simple_args(args, true)
    }

    fn format_deop_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // deop <targets>
        self.format_simple_args(args, true)
    }

    fn format_debug_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // debug start|stop|report
        self.format_simple_args(args, true)
    }

    fn format_publish_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // publish [<port>]
        self.format_simple_args(args, true)
    }

    fn format_spectate_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // spectate [<target>]
        self.format_simple_args(args, true)
    }

    fn format_jigsaw_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // jigsaw <subcommand>
        self.format_simple_args(args, true)
    }

    fn format_chunk_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // chunk <subcommand>
        self.format_simple_args(args, true)
    }

    fn format_help_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // help [<command>]
        self.format_simple_args(args, true)
    }

    fn format_datapack_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // datapack enable|disable|list [<name>]
        self.format_simple_args(args, true)
    }

    fn format_save_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // save all|query|hold
        self.format_simple_args(args, true)
    }

    fn format_ride_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // ride <targets> mount|dismount [<vehicle>]
        self.format_simple_args(args, true)
    }

    fn format_rotate_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // rotate <targets> <rotation>
        self.format_simple_args(args, true)
    }

    fn format_inputpermission_args(&self, args: &[CommandArg]) -> Result<String, String> {
        // inputpermission query|set <targets> <permission> [state]
        self.format_simple_args(args, true)
    }

    // === END NEW ARGUMENT FORMATTERS ===

    fn generate_if_block(&mut self, condition: &IfCondition, body: &Block) -> Result<String, String> {
        self.if_counter += 1;
        let block_name = format!("ifBlocks/if_{}", self.if_counter);
        
        // Generate the if block function
        let mut body_lines = Vec::new();
        for stmt in &body.statements {
            body_lines.push(self.generate_statement(stmt)?);
        }
        
        let body_content = body_lines.join("\n");
        self.functions.push(GeneratedFile {
            path: format!("{}.mcfunction", block_name),
            content: body_content,
        });
        
        // Generate the execute command
        let target = self.format_expr(&condition.target)?;
        let condition_str = self.format_condition(condition, &target)?;
        
        Ok(format!("execute {} run function {}:{}", condition_str, self.namespace, block_name))
    }
    
    fn format_condition(&self, condition: &IfCondition, target: &str) -> Result<String, String> {
        match condition.check_type.as_str() {
            "entity" => Ok(format!("if entity {}", target)),
            "block" => Ok(format!("if block {} {}", target, condition.operator)),
            "score" => Ok(format!("if score {}", target)),
            _ => Ok(format!("if entity {}", target)),
        }
    }
    
    /// Generate tag JSON files
    pub fn generate_tags(&self) -> Vec<(String, String)> {
        let mut tags = Vec::new();

        if !self.load_functions.is_empty() {
            let load_json = serde_json::json!({
                "values": self.load_functions.iter()
                    .map(|f| format!("{}:{}", self.namespace, f))
                    .collect::<Vec<_>>()
            });
            tags.push((
                "load.json".to_string(),
                serde_json::to_string_pretty(&load_json).unwrap(),
            ));
        }

        if !self.tick_functions.is_empty() {
            let tick_json = serde_json::json!({
                "values": self.tick_functions.iter()
                    .map(|f| format!("{}:{}", self.namespace, f))
                    .collect::<Vec<_>>()
            });
            tags.push((
                "tick.json".to_string(),
                serde_json::to_string_pretty(&tick_json).unwrap(),
            ));
        }

        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coord_formatting() {
        let gen = CodeGenerator::new("test");
        
        let coords = Coords {
            x: CoordValue::Relative(None),
            y: CoordValue::Relative(Some(1.0)),
            z: CoordValue::Relative(None),
        };
        
        assert_eq!(gen.format_coords(&coords), "~ ~1 ~");
    }
    
    #[test]
    fn test_command_mapping() {
        let gen = CodeGenerator::new("test");
        
        assert_eq!(gen.command_name_to_vanilla("tp"), "tp");
        assert_eq!(gen.command_name_to_vanilla("teleport"), "tp");
        assert_eq!(gen.command_name_to_vanilla("say"), "say");
        assert_eq!(gen.command_name_to_vanilla("gm"), "gamemode");
        assert_eq!(gen.command_name_to_vanilla("xp"), "xp");
    }
    
    #[test]
    fn test_tellraw_formatting() {
        let gen = CodeGenerator::new("test");
        
        let items = vec![
            Expr::String("Hello".to_string()),
            Expr::String("red".to_string()),
            Expr::Bool(false),
            Expr::Bool(true),
        ];
        
        let result = gen.format_tellraw_message(&items).unwrap();
        assert!(result.contains("\"text\":\"Hello\""));
        assert!(result.contains("\"color\":\"red\""));
        assert!(result.contains("\"bold\":true"));
    }
}
