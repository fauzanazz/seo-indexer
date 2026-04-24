use std::path::Path;
use tokio::fs;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum BulkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn read_urls_from_file(path: &Path) -> Result<Vec<Url>, BulkError> {
    let content = fs::read_to_string(path).await?;
    let urls = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .filter_map(|line| Url::parse(line.trim()).ok())
        .collect();
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_bulk_read_urls_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "https://example.com/page1").unwrap();
        writeln!(file, "https://example.com/page2").unwrap();

        let urls = read_urls_from_file(file.path()).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].as_str(), "https://example.com/page1");
        assert_eq!(urls[1].as_str(), "https://example.com/page2");
    }

    #[tokio::test]
    async fn test_bulk_skip_comments_and_empty() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "https://example.com/real").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "  # indented comment").unwrap();
        writeln!(file, "not-a-valid-url").unwrap();
        writeln!(file, "https://example.com/another").unwrap();

        let urls = read_urls_from_file(file.path()).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].as_str(), "https://example.com/real");
        assert_eq!(urls[1].as_str(), "https://example.com/another");
    }
}
