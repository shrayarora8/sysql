use crate::ast::*;
use std::io::IsTerminal;

// colors are ANSI escape codes, used only when a person is watching the terminal
fn paint(code: &str, text: &str) -> String {
    if std::io::stdout().is_terminal() && std::io::stderr().is_terminal() {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}

pub fn red(text: &str) -> String {
    paint("31", text)
}

pub fn green(text: &str) -> String {
    paint("32", text)
}

pub fn yellow(text: &str) -> String {
    paint("33", text)
}

pub fn cyan(text: &str) -> String {
    paint("36", text)
}

pub fn info(message: &str) {
    eprintln!("{} {}", cyan("●"), message);
}

pub fn ok(message: &str) {
    eprintln!("{} {}", green("✓"), message);
}

pub fn warn(message: &str) {
    eprintln!("{} {}", yellow("⚠"), yellow(message));
}

pub fn error(message: &str) {
    eprintln!("{} {}", red("✗"), message);
}

// unix seconds to "2026-11-22 18:40:02" (UTC)
pub fn clock(secs: u64) -> String {
    let (year, month, day) = civil_from_days((secs / 86400) as i64);
    let rest = secs % 86400;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year,
        month,
        day,
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

// days since 1970-01-01 to (year, month, day), Howard Hinnant's civil_from_days
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn pretty_cell(column: &str, cell: &str) -> String {
    match (column, cell.parse::<u64>()) {
        ("time", Ok(secs)) => clock(secs),
        _ => cell.to_string(),
    }
}

fn border(widths: &[usize], left: &str, middle: &str, right: &str) -> String {
    let mut parts = Vec::new();
    for width in widths {
        parts.push("─".repeat(width + 2));
    }
    dim(&format!("{}{}{}", left, parts.join(middle), right))
}

fn line(cells: &[String], widths: &[usize], header: bool) -> String {
    let mut parts = Vec::new();
    for (cell, width) in cells.iter().zip(widths) {
        let padded = format!(" {:<width$} ", cell, width = width);
        parts.push(if header { bold(&padded) } else { padded });
    }
    let bar = dim("│");
    format!("{}{}{}", bar, parts.join(&bar), bar)
}

// draws a table with box lines, like psql
pub fn print_table(table: &Table) {
    let mut rows = Vec::new();
    for row in &table.rows {
        let mut cells = Vec::new();
        for (i, cell) in row.iter().enumerate() {
            cells.push(pretty_cell(&table.columns[i], cell));
        }
        rows.push(cells);
    }

    let mut widths = Vec::new();
    for (i, column) in table.columns.iter().enumerate() {
        let mut width = column.chars().count();
        for row in &rows {
            width = width.max(row[i].chars().count());
        }
        widths.push(width);
    }

    println!("{}", border(&widths, "┌", "┬", "┐"));
    println!("{}", line(&table.columns, &widths, true));
    println!("{}", border(&widths, "├", "┼", "┤"));
    for row in &rows {
        println!("{}", line(row, &widths, false));
    }
    println!("{}", border(&widths, "└", "┴", "┘"));
    let n = rows.len();
    println!(
        "{}",
        dim(&format!("({} {})", n, if n == 1 { "row" } else { "rows" }))
    );
}

// a Record written back as SysQL text
pub fn rule_text(rule: &Record) -> String {
    let mut text = format!("RECORD {}", rule.table);
    if !rule.conds.is_empty() {
        let mut conds = Vec::new();
        for c in &rule.conds {
            if c.value.parse::<u64>().is_ok() {
                conds.push(format!("{} = {}", c.column, c.value));
            } else {
                conds.push(format!("{} = '{}'", c.column, c.value));
            }
        }
        text.push_str(&format!(" WHERE {}", conds.join(" AND ")));
    }
    format!("{} FOR {}", text, rule.seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        assert_eq!(clock(0), "1970-01-01 00:00:00");
        assert_eq!(clock(1795390635), "2026-11-22 23:37:15");
        assert_eq!(clock(951782400), "2000-02-29 00:00:00");
    }

    #[test]
    fn test_rule_text() {
        let rule = Record {
            table: "processes".to_string(),
            conds: vec![
                Cond {
                    column: "name".to_string(),
                    value: "curl".to_string(),
                },
                Cond {
                    column: "pid".to_string(),
                    value: "42".to_string(),
                },
            ],
            seconds: 30,
        };
        assert_eq!(
            rule_text(&rule),
            "RECORD processes WHERE name = 'curl' AND pid = 42 FOR 30"
        );
    }
}
