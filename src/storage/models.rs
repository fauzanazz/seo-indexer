use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRecord {
    pub id: Option<i64>,
    pub url: String,
    pub method: String,
    pub success: bool,
    pub message: Option<String>,
    pub submitted_at: DateTime<Utc>,
}
