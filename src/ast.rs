#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub time: u64,
    pub pid: u32,
    pub name: String,
    pub user: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cond {
    pub column: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub table: String,
    pub conds: Vec<Cond>,
    pub seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub columns: Vec<String>,
    pub table: String,
    pub conds: Vec<Cond>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordRun {
    pub rule: Record,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Record(Record),
    Select(Select),
}
