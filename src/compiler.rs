use crate::ast::*;

const PRINT: &str = r#"{ printf("%s %d %d %s\n", strftime("%s", nsecs), pid, uid, comm); }"#;

fn bpftrace_name(column: &str) -> &str {
    match column {
        "name" => "comm",
        "pid" => "pid",
        "user" => "uid",
        "time" => "nsecs",
        _ => unreachable!(),
    }
}

fn cond_text(c: &Cond) -> String {
    // name == "curl" becomes comm == "curl"
    let bpf_col_name = bpftrace_name(&c.column);
    if c.column == "name" {
        format!("{} == \"{}\"", bpf_col_name, c.value)
    } else if c.column == "user" {
        format!("{} == {}", bpf_col_name, user_id(&c.value))
    } else {
        format!("{} == {}", bpf_col_name, c.value)
    }
}

// the kernel only knows user ids, so a name like 'root' is looked up in /etc/passwd
fn user_id(user: &str) -> u32 {
    if let Ok(id) = user.parse() {
        return id;
    }
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap_or_default();
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() > 2 && fields[0] == user {
            return fields[2].parse().unwrap_or(u32::MAX);
        }
    }
    // no such user, and no real user has this id, so nothing matches
    u32::MAX
}

fn where_filter_text(conds: &[Cond]) -> String {
    let mut filters = Vec::new();
    for c in conds {
        filters.push(cond_text(c));
    }
    filters.join(" && ")
}

pub fn compile(rule: &Record) -> String {
    let conds = where_filter_text(&rule.conds);
    let filter = if conds.is_empty() {
        String::new()
    } else {
        format!(" /{}/", conds)
    };
    format!(
        "tracepoint:sched:sched_process_exec{} {}\ninterval:s:{} {{ exit(); }}\n",
        filter, PRINT, rule.seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(column: &str, value: &str) -> Cond {
        Cond {
            column: column.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn test_cond_text() {
        assert_eq!(cond_text(&cond("name", "curl")), "comm == \"curl\"");
        assert_eq!(cond_text(&cond("pid", "42")), "pid == 42");
    }

    #[test]
    fn test_user() {
        assert_eq!(cond_text(&cond("user", "501")), "uid == 501");
        assert_eq!(cond_text(&cond("user", "root")), "uid == 0");
        assert_eq!(cond_text(&cond("user", "nobody_here")), "uid == 4294967295");
    }

    #[test]
    fn test_where_filter_text() {
        assert_eq!(where_filter_text(&[]), "");
        assert_eq!(
            where_filter_text(&[cond("name", "curl")]),
            "comm == \"curl\""
        );
        assert_eq!(
            where_filter_text(&[cond("name", "curl"), cond("pid", "42")]),
            "comm == \"curl\" && pid == 42"
        );
    }

    #[test]
    fn test_compile() {
        let rule = Record {
            table: "processes".to_string(),
            conds: vec![cond("name", "curl"), cond("pid", "42")],
            seconds: 30,
        };
        assert_eq!(
            compile(&rule),
            r#"tracepoint:sched:sched_process_exec /comm == "curl" && pid == 42/ { printf("%s %d %d %s\n", strftime("%s", nsecs), pid, uid, comm); }
interval:s:30 { exit(); }
"#
        );
    }

    #[test]
    fn test_compile_no_where() {
        let rule = Record {
            table: "processes".to_string(),
            conds: vec![],
            seconds: 10,
        };
        assert_eq!(
            compile(&rule),
            r#"tracepoint:sched:sched_process_exec { printf("%s %d %d %s\n", strftime("%s", nsecs), pid, uid, comm); }
interval:s:10 { exit(); }
"#
        );
    }
}
