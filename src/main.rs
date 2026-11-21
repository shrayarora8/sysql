mod analyzer;
mod ast;
mod compiler;
mod interpreter;
mod parser;
mod recorder;

use ast::*;

fn print_table(table: &Table) {
    println!("{}", table.columns.join("\t"));
    for row in &table.rows {
        println!("{}", row.join("\t"));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: sysql \"<statement>\"");
        return;
    }
    let text = &args[1];

    match parser::parse(text) {
        None => eprintln!("could not parse: {}", text),
        Some(Stmt::Record(rule)) => {
            let script = compiler::compile(&rule);
            recorder::record(text, &script);
        }
        Some(Stmt::Select(query)) => {
            let history = recorder::read_history();
            if let Some(warning) = analyzer::analyze(&history, &query) {
                println!("WARNING: {}", warning);
            }
            let rows = recorder::read_log();
            print_table(&interpreter::run(&query, &rows));
        }
    }
}
