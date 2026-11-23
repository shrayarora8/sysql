mod analyzer;
mod ast;
mod compiler;
mod display;
mod interpreter;
mod parser;
mod recorder;
mod repl;

use ast::*;

// runs one statement, from the shell or from the command line
pub fn execute(text: &str) {
    let text = text.trim().trim_end_matches(';').trim();
    match parser::parse(text) {
        None => display::error(&parser::diagnose(text)),
        Some(Stmt::Record(rule)) => {
            if !cfg!(target_os = "linux") {
                display::error("RECORD needs Linux with bpftrace, run it inside the VM");
                return;
            }
            let script = compiler::compile(&rule);
            recorder::record(text, &script);
        }
        Some(Stmt::Select(query)) => {
            let history = recorder::read_history();
            if let Some(warning) = analyzer::analyze(&history, &query) {
                display::warn(&warning);
            }
            let rows = recorder::read_log();
            display::print_table(&interpreter::run(&query, &rows));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => repl::start(),
        2 => execute(&args[1]),
        _ => eprintln!("usage: sysql            (interactive shell)\n       sysql \"<statement>\""),
    }
}
