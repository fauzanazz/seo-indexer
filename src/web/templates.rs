use crate::storage::SubmissionRecord;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub recent: Vec<SubmissionRecord>,
    pub total_submissions: usize,
    pub success_rate: String,
}

#[derive(Template)]
#[template(path = "history.html")]
pub struct HistoryTemplate {
    pub submissions: Vec<SubmissionRecord>,
}

#[derive(Template)]
#[template(path = "submit.html")]
pub struct SubmitTemplate;

#[derive(Template)]
#[template(path = "result.html")]
pub struct ResultTemplate {
    pub success: bool,
    pub method: String,
    pub message: String,
}
