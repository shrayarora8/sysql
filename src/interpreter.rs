use crate::ast::*;

fn field(row: &Row, column: &str) -> String {
    match column {
        "time" => row.time.to_string(),
        "pid" => row.pid.to_string(),
        "name" => row.name.clone(),
        "user" => row.user.clone(),
        _ => unreachable!(),
    }
}

fn matches(row: &Row, conds: &[Cond]) -> bool {
    for c in conds {
        if field(row, &c.column) != c.value {
            return false;
        }
    }
    true
}

pub fn run(query: &Select, rows: &[Row]) -> Table {
    let mut out = Vec::new();
    for row in rows {
        if matches(row, &query.conds) {
            let mut cells = Vec::new();
            for column in &query.columns {
                cells.push(field(row, column));
            }
            out.push(cells);
        }
    }
    Table {
        columns: query.columns.clone(),
        rows: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(time: u64, pid: u32, name: &str, user: &str) -> Row {
        Row {
            time,
            pid,
            name: name.to_string(),
            user: user.to_string(),
        }
    }

    fn rows() -> Vec<Row> {
        vec![
            row(1795390635, 10, "curl", "shrayarora"),
            row(1795390636, 11, "ls", "shrayarora"),
            row(1795390640, 12, "curl", "root"),
        ]
    }

    fn select(columns: &[&str], conds: &[(&str, &str)]) -> Select {
        let mut c = Vec::new();
        for (column, value) in conds {
            c.push(Cond {
                column: column.to_string(),
                value: value.to_string(),
            });
        }
        let mut cols = Vec::new();
        for column in columns {
            cols.push(column.to_string());
        }
        Select {
            columns: cols,
            table: "processes".to_string(),
            conds: c,
        }
    }

    // the example from the lesson: filter on name, keep two columns
    #[test]
    fn test_filter_and_pick() {
        let t = run(&select(&["pid", "user"], &[("name", "curl")]), &rows());
        assert_eq!(t.columns, vec!["pid", "user"]);
        assert_eq!(t.rows, vec![vec!["10", "shrayarora"], vec!["12", "root"]]);
    }

    // no WHERE keeps every row
    #[test]
    fn test_no_where() {
        assert_eq!(run(&select(&["name"], &[]), &rows()).rows.len(), 3);
    }

    // every condition has to hold
    #[test]
    fn test_and() {
        let t = run(
            &select(&["pid"], &[("name", "curl"), ("user", "root")]),
            &rows(),
        );
        assert_eq!(t.rows, vec![vec!["12"]]);
    }

    // numbers are compared as text, and nothing matching gives an empty table
    #[test]
    fn test_pid_and_empty() {
        assert_eq!(
            run(&select(&["name"], &[("pid", "11")]), &rows()).rows,
            vec![vec!["ls"]]
        );
        assert!(
            run(&select(&["name"], &[("name", "bash")]), &rows())
                .rows
                .is_empty()
        );
    }
}
