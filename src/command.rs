#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Analyze {
        sql: String,
        plan: bool,
        runtime: bool,
        raw_plan: bool,
        format: AnalyzeFormat,
    },
    LabList,
    LabRun {
        scenario: String,
    },
    LabSeed {
        profile: String,
    },
    Add {
        title: String,
    },
    List,
    Done {
        id: i64,
    },
    Delete {
        id: i64,
    },
    Search {
        keyword: String,
    },
    Stats,
    ProjectAdd {
        name: String,
    },
    ProjectList,
    ProjectShow {
        id: i64,
    },
    ProjectDelete {
        id: i64,
    },
    ProjectStats {
        id: Option<i64>,
    },
    TaskAdd {
        project_id: Option<i64>,
        priority: i64,
        title: String,
        tags: Vec<String>,
    },
    TaskList {
        project_id: Option<i64>,
        tag: Option<String>,
    },
    TaskShow {
        id: i64,
    },
    TaskDone {
        id: i64,
    },
    TaskDelete {
        id: i64,
    },
    TaskSearch {
        keyword: String,
    },
    TaskTag {
        id: i64,
        tag: String,
    },
    TaskUntag {
        id: i64,
        tag: String,
    },
    TaskTags {
        id: i64,
    },
    TagAdd {
        name: String,
    },
    TagList,
    TagDelete {
        id: i64,
    },
    Seed,
    Sql {
        sql: String,
    },
    Repl,
    Help,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalyzeFormat {
    Tree,
    Json,
}
