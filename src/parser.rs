use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Str(String), // things in quotes like where name = 'curl'
    Num(u64),
    Equals,
    Comma,
    Star,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(token)
    }

    fn peek(&self) -> Option<Token> {
        let token = self.tokens.get(self.pos)?.clone();
        Some(token)
    }

    fn expect(&mut self, want: Token) -> Option<()> {
        if self.next()? == want { Some(()) } else { None }
    }

    fn word(&mut self) -> Option<String> {
        match self.next()? {
            Token::Word(w) => Some(w),
            _ => None,
        }
    }

    fn num(&mut self) -> Option<u64> {
        match self.next()? {
            Token::Num(n) => Some(n),
            _ => None,
        }
    }

    fn value(&mut self) -> Option<String> {
        match self.next()? {
            Token::Str(w) => Some(w),
            Token::Num(n) => Some(n.to_string()),
            _ => None,
        }
    }

    fn parse_cond(&mut self) -> Option<Cond> {
        let column = self.word()?;
        self.expect(Token::Equals)?;
        let value = self.value()?;
        let condition = Cond { column, value };
        Some(condition)
    }

    fn parse_conds(&mut self) -> Option<Vec<Cond>> {
        let mut conds = Vec::new();
        conds.push(self.parse_cond()?);
        while self.peek() == Some(Token::Word("AND".to_string())) {
            self.next();
            conds.push(self.parse_cond()?);
        }
        Some(conds)
    }

    fn parse_columns(&mut self) -> Option<Vec<String>> {
        if self.peek() == Some(Token::Star) {
            self.next();
            Some(vec![
                "time".to_string(),
                "pid".to_string(),
                "name".to_string(),
                "user".to_string(),
            ])
        } else {
            let mut columns = Vec::new();
            let mut column = self.word()?;
            columns.push(column);
            while self.peek() == Some(Token::Comma) {
                self.next();
                column = self.word()?;
                columns.push(column);
            }
            Some(columns)
        }
    }

    fn parse_record(&mut self) -> Option<Record> {
        self.expect(Token::Word("RECORD".to_string()))?;
        let table = self.word()?;
        let mut conds = Vec::new();
        if self.peek() == Some(Token::Word("WHERE".to_string())) {
            self.next();
            conds = self.parse_conds()?;
        }
        self.expect(Token::Word("FOR".to_string()))?;
        let seconds = self.num()?;
        Some(Record {
            table,
            conds,
            seconds,
        })
    }

    fn parse_select(&mut self) -> Option<Select> {
        self.expect(Token::Word("SELECT".to_string()))?;
        let columns = self.parse_columns()?;
        self.expect(Token::Word("FROM".to_string()))?;
        let table = self.word()?;
        let mut conds = Vec::new();
        if self.peek() == Some(Token::Word("WHERE".to_string())) {
            self.next();
            conds = self.parse_conds()?;
        }
        Some(Select {
            columns,
            table,
            conds,
        })
    }
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == ',' {
            tokens.push(Token::Comma);
            i += 1;
        } else if c == '=' {
            tokens.push(Token::Equals);
            i += 1;
        } else if c == '*' {
            tokens.push(Token::Star);
            i += 1;
        } else if c.is_alphabetic() {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::Word(word));
        } else if c.is_ascii_digit() {
            let mut word = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                word.push(chars[i]);
                i += 1;
            }
            let num = word.parse::<u64>().ok()?;
            tokens.push(Token::Num(num))
        } else if c == '\'' {
            i += 1;
            let mut word = String::new();
            while i < chars.len() && chars[i] != '\'' {
                word.push(chars[i]);
                i += 1;
            }
            if i == chars.len() {
                // ran off the end, so the quote was never closed
                return None;
            }
            i += 1;
            tokens.push(Token::Str(word))
        } else {
            return None;
        }
    }
    Some(tokens)
}

const COLUMNS: [&str; 4] = ["time", "pid", "name", "user"];

pub fn parse(text: &str) -> Option<Stmt> {
    let tokens = tokenize(text)?;
    let mut p = Parser { tokens, pos: 0 };

    let stmt = if p.peek() == Some(Token::Word("RECORD".to_string())) {
        Stmt::Record(p.parse_record()?)
    } else {
        Stmt::Select(p.parse_select()?)
    };

    if p.pos != p.tokens.len() {
        return None;
    }

    let ok = match &stmt {
        Stmt::Record(r) => known_names(&r.table, &r.conds, &[]),
        Stmt::Select(s) => known_names(&s.table, &s.conds, &s.columns),
    };
    if !ok {
        return None;
    }

    Some(stmt)
}

fn known_names(table: &str, conds: &[Cond], columns: &[String]) -> bool {
    if table != "processes" {
        return false;
    }
    for c in conds {
        if !COLUMNS.contains(&c.column.as_str()) {
            return false;
        }
    }
    for c in columns {
        if !COLUMNS.contains(&c.as_str()) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize("RECORD processes WHERE name = 'curl' FOR 60"),
            Some(vec![
                Word("RECORD".to_string()),
                Word("processes".to_string()),
                Word("WHERE".to_string()),
                Word("name".to_string()),
                Equals,
                Str("curl".to_string()),
                Word("FOR".to_string()),
                Num(60),
            ])
        );
        assert_eq!(tokenize("name = 'oops"), None);
        assert_eq!(tokenize("pid = 4; ls"), None);
    }

    #[test]
    fn test_helpers() {
        let mut p = Parser {
            tokens: tokenize("FROM processes = 'curl' 60").unwrap(),
            pos: 0,
        };
        assert_eq!(p.expect(Token::Word("FROM".to_string())), Some(()));
        assert_eq!(p.word(), Some("processes".to_string()));
        assert_eq!(p.expect(Token::Equals), Some(()));
        assert_eq!(p.value(), Some("curl".to_string()));
        assert_eq!(p.num(), Some(60));
        assert_eq!(p.num(), None);
    }

    #[test]
    fn test_conds() {
        let mut p = Parser {
            tokens: tokenize("name = 'curl' AND pid = 42 FOR 60").unwrap(),
            pos: 0,
        };
        assert_eq!(
            p.parse_conds(),
            Some(vec![
                Cond {
                    column: "name".to_string(),
                    value: "curl".to_string()
                },
                Cond {
                    column: "pid".to_string(),
                    value: "42".to_string()
                },
            ])
        );
        assert_eq!(p.peek(), Some(Token::Word("FOR".to_string())));

        let mut bad = Parser {
            tokens: tokenize("name = AND").unwrap(),
            pos: 0,
        };
        assert_eq!(bad.parse_conds(), None);
    }
}

#[test]
fn test_record_select() {
    let mut p = Parser {
        tokens: tokenize("RECORD processes FOR 60").unwrap(),
        pos: 0,
    };
    assert_eq!(
        p.parse_record(),
        Some(Record {
            table: "processes".to_string(),
            conds: vec![],
            seconds: 60
        })
    );

    let mut p = Parser {
        tokens: tokenize("SELECT pid, name FROM processes WHERE user = 'root'").unwrap(),
        pos: 0,
    };
    assert_eq!(
        p.parse_select(),
        Some(Select {
            columns: vec!["pid".to_string(), "name".to_string()],
            table: "processes".to_string(),
            conds: vec![Cond {
                column: "user".to_string(),
                value: "root".to_string()
            }],
        })
    );

    let mut p = Parser {
        tokens: tokenize("SELECT * FROM processes").unwrap(),
        pos: 0,
    };
    assert_eq!(p.parse_select().unwrap().columns.len(), 4);

    let mut p = Parser {
        tokens: tokenize("RECORD processes WHERE name = 'curl'").unwrap(),
        pos: 0,
    };
    assert_eq!(p.parse_record(), None);
}

#[cfg(test)]
mod edge_tests {
    use super::*;

    fn parser(text: &str) -> Parser {
        Parser {
            tokens: tokenize(text).unwrap(),
            pos: 0,
        }
    }

    // a WHERE, AND, or comma with nothing after it is rejected
    #[test]
    fn test_dangling() {
        assert_eq!(
            parser("SELECT pid FROM processes WHERE").parse_select(),
            None
        );
        assert_eq!(
            parser("SELECT pid FROM processes WHERE name = 'curl' AND").parse_select(),
            None
        );
        assert_eq!(parser("SELECT pid, FROM processes").parse_select(), None);
        assert_eq!(parser("RECORD processes WHERE FOR 60").parse_record(), None);
    }

    // keywords must be in caps and in the right order, and spacing does not matter
    #[test]
    fn test_order_and_spacing() {
        assert_eq!(parser("record processes for 60").parse_record(), None);
        assert_eq!(parser("RECORD FOR 60").parse_record(), None);
        assert_eq!(parser("SELECT FROM processes").parse_select(), None);
        assert_eq!(
            parser("  SELECT   pid,name FROM processes WHERE pid = 42 ").parse_select(),
            Some(Select {
                columns: vec!["pid".to_string(), "name".to_string()],
                table: "processes".to_string(),
                conds: vec![Cond {
                    column: "pid".to_string(),
                    value: "42".to_string()
                }],
            })
        );
    }

    // parse() has to reject leftover tokens and names that don't exist
    #[test]
    fn test_parse_step5() {
        assert_eq!(parse("SELECT pid FROM processes banana"), None);
        assert_eq!(parse("RECORD processes FOR 60 60"), None);
        assert_eq!(parse("RECORD bananas FOR 60"), None);
        assert_eq!(parse("SELECT color FROM processes"), None);
        assert_eq!(parse("SELECT pid FROM processes WHERE shoe = 'red'"), None);
        assert_eq!(parse(""), None);
        assert!(
            parse("RECORD processes WHERE name = 'curl' AND user = 'root' FOR 86400").is_some()
        );
    }
}
