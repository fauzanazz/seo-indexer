use quick_xml::events::Event;
use quick_xml::Reader;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum SitemapError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

pub struct SitemapParser;

impl SitemapParser {
    /// Parse sitemap XML and extract all URLs.
    /// Handles both regular sitemaps (`<url><loc>`) and sitemap indexes (`<sitemap><loc>`).
    pub fn parse(xml: &str) -> Result<Vec<Url>, SitemapError> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut urls = Vec::new();
        let mut inside_loc = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref tag)) if tag.name().as_ref() == b"loc" => {
                    inside_loc = true;
                }
                Ok(Event::Text(text)) if inside_loc => {
                    let raw = text
                        .unescape()
                        .map_err(|e| SitemapError::Xml(e.to_string()))?;
                    let trimmed = raw.trim();
                    let url = Url::parse(trimmed)
                        .map_err(|e| SitemapError::InvalidUrl(format!("{trimmed}: {e}")))?;
                    urls.push(url);
                    inside_loc = false;
                }
                Ok(Event::End(_)) => {
                    inside_loc = false;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(SitemapError::Xml(e.to_string())),
                _ => {}
            }
        }

        Ok(urls)
    }

    /// Fetch and parse a sitemap from a URL.
    /// If the sitemap is an index, recursively fetches and merges child sitemaps.
    pub async fn fetch_and_parse(sitemap_url: &Url) -> Result<Vec<Url>, SitemapError> {
        let xml = reqwest::get(sitemap_url.as_str()).await?.text().await?;
        let found = Self::parse(&xml)?;

        // If every found URL looks like a sitemap itself, treat as sitemap index
        let child_sitemaps: Vec<&Url> = found
            .iter()
            .filter(|u| {
                let path = u.path().to_lowercase();
                path.ends_with(".xml") || path.contains("sitemap")
            })
            .collect();

        if !child_sitemaps.is_empty() && child_sitemaps.len() == found.len() {
            let mut all_urls = Vec::new();
            for child_url in &found {
                match Box::pin(Self::fetch_and_parse(child_url)).await {
                    Ok(child_urls) => all_urls.extend(child_urls),
                    Err(_) => all_urls.push(child_url.clone()),
                }
            }
            return Ok(all_urls);
        }

        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sitemap_parse_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://example.com/page1</loc>
        <lastmod>2024-01-01</lastmod>
    </url>
    <url>
        <loc>https://example.com/page2</loc>
    </url>
</urlset>"#;

        let urls = SitemapParser::parse(xml).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].as_str(), "https://example.com/page1");
        assert_eq!(urls[1].as_str(), "https://example.com/page2");
    }

    #[test]
    fn test_sitemap_parse_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <sitemap>
        <loc>https://example.com/sitemap-posts.xml</loc>
    </sitemap>
    <sitemap>
        <loc>https://example.com/sitemap-pages.xml</loc>
    </sitemap>
</sitemapindex>"#;

        let urls = SitemapParser::parse(xml).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].as_str(), "https://example.com/sitemap-posts.xml");
        assert_eq!(urls[1].as_str(), "https://example.com/sitemap-pages.xml");
    }
}
