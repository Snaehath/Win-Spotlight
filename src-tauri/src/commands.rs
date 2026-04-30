/// Command Plugin System — Extensible registry for `> prefix` commands.
///
/// Usage examples:
///   > calc 10 + 20           — Evaluate a math expression
///   > g rust ownership       — Web search via browser
///   > open notepad           — Launch a named system utility
///   > sys sleep              — System power actions

use std::process::Command as SysCommand;
use std::os::windows::process::CommandExt;

// ── Trait ──────────────────────────────────────────────────────────────────

pub trait CommandPlugin: Send + Sync {
    /// The primary prefix keyword (e.g. "calc", "g", "open", "sys")
    fn prefix(&self) -> &str;
    /// Human-readable short description shown in the UI
    fn description(&self) -> &str;
    /// Execute the command and return a displayable result string (or action confirmation)
    fn execute(&self, args: &str) -> CommandResult;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CommandResult {
    /// Show the string in the UI result row
    Display(String),
    /// Launch an external process (path to exe and args) — no display
    Launch(String, Vec<String>),
    /// Nothing to show (e.g. system action kicked off)
    Silent,
    /// Something went wrong
    Error(String),
}

// ── Registry ───────────────────────────────────────────────────────────────

pub struct CommandRegistry {
    plugins: Vec<Box<dyn CommandPlugin>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut reg = CommandRegistry { plugins: Vec::new() };
        reg.register(Box::new(CalcPlugin));
        reg.register(Box::new(WebSearchPlugin));
        reg.register(Box::new(SysPlugin));
        reg
    }

    pub fn register(&mut self, plugin: Box<dyn CommandPlugin>) {
        self.plugins.push(plugin);
    }

    /// Parse a raw query starting with `>`, route to the matching plugin.
    ///
    /// Returns `None` if the query doesn't start with `>` or no plugin matches.
    pub fn handle(&self, raw_query: &str) -> Option<CommandResult> {
        let trimmed = raw_query.trim();
        if !trimmed.starts_with('>') {
            return None;
        }

        // Strip the `>` and leading spaces
        let body = trimmed.trim_start_matches('>').trim();
        let (prefix, args) = body.split_once(' ').unwrap_or((body, ""));

        for plugin in &self.plugins {
            if plugin.prefix().eq_ignore_ascii_case(prefix) {
                return Some(plugin.execute(args.trim()));
            }
        }

        Some(CommandResult::Error(format!(
            "Unknown command '{}'. Try: calc, g, sys",
            prefix
        )))
    }

    pub fn all_hints(&self) -> Vec<(&str, &str)> {
        self.plugins.iter().map(|p| (p.prefix(), p.description())).collect()
    }
}

// ── Plugins ────────────────────────────────────────────────────────────────

/// > calc <expression>  — Basic math via Windows Calculator for complex math;
///   simple expressions (+ - * /) are evaluated inline.
struct CalcPlugin;
impl CommandPlugin for CalcPlugin {
    fn prefix(&self) -> &str { "calc" }
    fn description(&self) -> &str { "Evaluate a math expression: > calc 10 * 3" }

    fn execute(&self, args: &str) -> CommandResult {
        if args.is_empty() {
            // Launch Calculator app
            return CommandResult::Launch("calc.exe".to_string(), vec![]);
        }
        // Safely evaluate simple integer-only expressions
        match eval_simple(args) {
            Some(result) => CommandResult::Display(format!("{} = {}", args, result)),
            None => CommandResult::Launch("calc.exe".to_string(), vec![]),
        }
    }
}

/// Simple safe evaluator for `a op b` style expressions (no exec/eval)
pub fn eval_simple(expr: &str) -> Option<f64> {
    let expr = expr.replace(' ', "");
    let ops = ['+', '-', '*', '/'];
    for op in ops {
        if let Some(pos) = expr.rfind(op) {
            if pos == 0 { continue; }
            let left: f64 = expr[..pos].parse().ok()?;
            let right: f64 = expr[pos + 1..].parse().ok()?;
            return Some(match op {
                '+' => left + right,
                '-' => left - right,
                '*' => left * right,
                '/' => if right == 0.0 { return None; } else { left / right },
                _ => return None,
            });
        }
    }
    None
}

/// > g <search terms>  — Open default browser with a Google search
struct WebSearchPlugin;
impl CommandPlugin for WebSearchPlugin {
    fn prefix(&self) -> &str { "g" }
    fn description(&self) -> &str { "Web search: > g rust ownership" }

    fn execute(&self, args: &str) -> CommandResult {
        if args.is_empty() {
            return CommandResult::Error("Usage: > g <search terms>".to_string());
        }
        let encoded = args.replace(' ', "+");
        let url = format!("https://www.google.com/search?q={}", encoded);
        // Safely launch URL via explorer/powershell instead of cmd.exe
        CommandResult::Launch("https".to_string(), vec![url]) // Tagged as https to help launcher.rs identify it
    }
}


/// > sys <action>  — System power/management actions
struct SysPlugin;
impl CommandPlugin for SysPlugin {
    fn prefix(&self) -> &str { "sys" }
    fn description(&self) -> &str { "System actions: > sys sleep | shutdown | restart | lock" }

    fn execute(&self, args: &str) -> CommandResult {
        match args.to_lowercase().as_str() {
            "sleep"    => CommandResult::Launch("rundll32.exe".to_string(), vec!["powrprof.dll,SetSuspendState".to_string(), "0,1,0".to_string()]),
            "shutdown" => CommandResult::Launch("shutdown.exe".to_string(), vec!["/s".to_string(), "/t".to_string(), "0".to_string()]),
            "restart"  => CommandResult::Launch("shutdown.exe".to_string(), vec!["/r".to_string(), "/t".to_string(), "0".to_string()]),
            "lock"     => CommandResult::Launch("rundll32.exe".to_string(), vec!["user32.dll,LockWorkStation".to_string()]),
            "exit"     => CommandResult::Launch("exit".to_string(), vec![]),
            _ => CommandResult::Error(format!(
                "Unknown sys action '{}'. Try: sleep, shutdown, restart, lock, exit", args
            )),
        }
    }
}


pub fn execute_command_result(result: CommandResult) -> Result<Option<String>, String> {
    match result {
        CommandResult::Display(s) => Ok(Some(s)),
        CommandResult::Silent => Ok(None),
        CommandResult::Error(e) => Err(e),
        CommandResult::Launch(exe, args) => {
            if exe == "https" {
                // Use native ShellExecuteW to open URLs securely.
                // This bypasses the shell (PowerShell/CMD) and is immune to injection.
                crate::shell::open_path_or_url(&args[0]).map_err(|e| e.to_string())?;
            } else if exe == "exit" {
                std::process::exit(0);
            } else {
                SysCommand::new(&exe)
                    .args(&args)
                    .creation_flags(0x08000000)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            Ok(None)
        }
    }
}
