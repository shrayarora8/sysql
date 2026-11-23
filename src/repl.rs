use crate::ast::*;
use crate::display::*;
use crate::recorder;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

const HELP: &str = "\
Statements
  RECORD processes [WHERE column = value [AND ...]] FOR seconds
  SELECT column, ... FROM processes [WHERE column = value [AND ...]]
  SELECT * FROM processes

Commands
  \\d          show the columns you can use
  \\history    show past recordings
  \\help       show this help
  \\q          quit

Examples
  RECORD processes WHERE name = 'curl' FOR 30
  SELECT time, pid, user FROM processes WHERE name = 'curl'
  SELECT * FROM processes WHERE user = 'root'";

fn show_columns() {
    let columns = [
        ("time", "when the program started (UTC)"),
        ("pid", "process id"),
        ("name", "program name, first 15 characters"),
        ("user", "who ran it"),
    ];
    let mut rows = Vec::new();
    for (column, meaning) in columns {
        rows.push(vec![column.to_string(), meaning.to_string()]);
    }
    print_table(&Table {
        columns: vec!["column".to_string(), "meaning".to_string()],
        rows,
    });
}

fn show_history() {
    let runs = recorder::read_history();
    if runs.is_empty() {
        info("no recordings yet, try: RECORD processes WHERE name = 'curl' FOR 30");
        return;
    }
    let mut rows = Vec::new();
    for (i, run) in runs.iter().enumerate() {
        rows.push(vec![
            (i + 1).to_string(),
            clock(run.start),
            clock(run.end),
            rule_text(&run.rule),
        ]);
    }
    print_table(&Table {
        columns: vec![
            "#".to_string(),
            "started".to_string(),
            "ended".to_string(),
            "rule".to_string(),
        ],
        rows,
    });
}

// the interactive shell: read a line, run it, repeat
pub fn start() {
    println!(
        "{} {}",
        bold("SysQL 0.1"),
        dim("· record and query Linux process events")
    );
    println!("{}", dim("Type \\help for help, \\q to quit."));

    let mut editor = DefaultEditor::new().expect("could not start the line editor");
    let history_file = format!(
        "{}/.sysql_history",
        std::env::var("HOME").unwrap_or_default()
    );
    let _ = editor.load_history(&history_file);

    loop {
        match editor.readline("sysql> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = editor.add_history_entry(line);
                match line {
                    "\\q" | "quit" | "exit" => break,
                    "\\help" | "\\h" | "\\?" | "help" => println!("{}", HELP),
                    "\\d" => show_columns(),
                    "\\history" => show_history(),
                    _ if line.starts_with('\\') => error("unknown command, try \\help"),
                    _ => crate::execute(line),
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                error(&e.to_string());
                break;
            }
        }
    }
    let _ = editor.save_history(&history_file);
}
