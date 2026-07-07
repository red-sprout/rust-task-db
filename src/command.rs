#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add { title: String },
    List,
    Done { id: i64 },
    Delete { id: i64 },
    Search { keyword: String },
    Stats,
    Sql { sql: String },
    Repl,
    Help,
}
