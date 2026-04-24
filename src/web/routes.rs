use super::{templates::*, AppState};
use askama::Template;
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use std::sync::Arc;

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/history", get(history))
        .route("/submit", get(submit_form))
        .route("/submit", post(submit_url))
        .with_state(state)
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let storage = state.storage.read().await;
    let recent = storage.get_history(10).unwrap_or_default();
    let all = storage.get_history(1000).unwrap_or_default();
    let total = all.len();
    let successes = all.iter().filter(|r| r.success).count();
    let rate = if total > 0 {
        (successes as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let template = IndexTemplate {
        recent,
        total_submissions: total,
        success_rate: format!("{:.1}", rate),
    };
    Html(template.render().unwrap_or_default())
}

async fn history(State(state): State<Arc<AppState>>) -> Html<String> {
    let storage = state.storage.read().await;
    let submissions = storage.get_history(100).unwrap_or_default();
    let template = HistoryTemplate { submissions };
    Html(template.render().unwrap_or_default())
}

async fn submit_form() -> Html<String> {
    Html(SubmitTemplate.render().unwrap_or_default())
}

#[derive(serde::Deserialize)]
pub struct SubmitForm {
    url: String,
    method: String,
}

async fn submit_url(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    use crate::storage::SubmissionRecord;
    use chrono::Utc;
    use url::Url;

    let url = match Url::parse(&form.url) {
        Ok(u) => u,
        Err(_) => {
            let t = ResultTemplate {
                success: false,
                method: form.method,
                message: "Invalid URL".into(),
            };
            return Html(t.render().unwrap_or_default());
        }
    };

    let indexers = crate::indexers::get_indexers(&state.config, &form.method);
    let mut results: Vec<(String, bool, String)> = Vec::new();

    for indexer in &indexers {
        let (success, message) = match indexer.submit(&url).await {
            Ok(r) => (r.success, r.message),
            Err(e) => (false, e.to_string()),
        };

        let record = SubmissionRecord {
            id: None,
            url: url.to_string(),
            method: indexer.name().to_string(),
            success,
            message: Some(message.clone()),
            submitted_at: Utc::now(),
        };

        let _ = state.storage.write().await.insert(&record);
        results.push((indexer.name().to_string(), success, message));
    }

    let (method, success, message) = results
        .into_iter()
        .next()
        .unwrap_or_else(|| (form.method.clone(), false, "No indexers configured".into()));

    Html(
        ResultTemplate {
            success,
            method,
            message,
        }
        .render()
        .unwrap_or_default(),
    )
}
