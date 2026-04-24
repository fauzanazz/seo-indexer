use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{Indexer, SubmissionResult};
use crate::error::IndexerError;

const GOOGLE_INDEXING_ENDPOINT: &str =
    "https://indexing.googleapis.com/v3/urlNotifications:publish";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const INDEXING_SCOPE: &str = "https://www.googleapis.com/auth/indexing";

pub struct GoogleIndexingClient {
    service_account_path: String,
    http: Client,
    token_endpoint: String,
    indexing_endpoint: String,
}

#[derive(Deserialize)]
struct ServiceAccount {
    client_email: String,
    private_key: String,
}

#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Serialize)]
struct IndexingRequest<'a> {
    url: &'a str,
    #[serde(rename = "type")]
    notification_type: &'a str,
}

impl GoogleIndexingClient {
    pub fn new(service_account_path: String) -> Self {
        Self {
            service_account_path,
            http: Client::new(),
            token_endpoint: GOOGLE_TOKEN_ENDPOINT.to_string(),
            indexing_endpoint: GOOGLE_INDEXING_ENDPOINT.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_endpoints(mut self, token_endpoint: String, indexing_endpoint: String) -> Self {
        self.token_endpoint = token_endpoint;
        self.indexing_endpoint = indexing_endpoint;
        self
    }

    async fn get_access_token(&self) -> Result<String, IndexerError> {
        let contents = std::fs::read_to_string(&self.service_account_path).map_err(|source| {
            IndexerError::FileRead {
                path: self.service_account_path.clone(),
                source,
            }
        })?;

        let account: ServiceAccount = serde_json::from_str(&contents)
            .map_err(|e| IndexerError::ServiceAccountParse(e.to_string()))?;

        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: account.client_email,
            scope: INDEXING_SCOPE.to_string(),
            aud: self.token_endpoint.clone(),
            iat: now,
            exp: now + 3600,
        };

        let encoding_key = EncodingKey::from_rsa_pem(account.private_key.as_bytes())
            .map_err(|e| IndexerError::JwtSign(e.to_string()))?;

        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
            .map_err(|e| IndexerError::JwtSign(e.to_string()))?;

        let response = self
            .http
            .post(&self.token_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown".to_string());
            return Err(IndexerError::AuthTokenExchange(message));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| IndexerError::AuthTokenExchange(e.to_string()))?;

        Ok(token_response.access_token)
    }
}

#[async_trait]
impl Indexer for GoogleIndexingClient {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn submit(&self, url: &Url) -> Result<SubmissionResult, IndexerError> {
        let access_token = self.get_access_token().await?;

        let body = IndexingRequest {
            url: url.as_str(),
            notification_type: "URL_UPDATED",
        };

        let response = self
            .http
            .post(&self.indexing_endpoint)
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            return Ok(SubmissionResult {
                success: true,
                method: "google".to_string(),
                message: format!("Submitted successfully (HTTP {})", status.as_u16()),
                timestamp: Utc::now(),
            });
        }

        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(IndexerError::Api {
            status: status.as_u16(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // 2048-bit RSA key in PKCS8 format, generated for testing only
    const TEST_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCuDMC0BX3GBp4E
Dm8p4HIUxjzfntz3kHTDCeviofcPDp9aJt/nq5X1mTFgvkZIprImK+EAJNU9ICU/
dakZR98ksIWKvVNjVQwhNcr7It0cviUq6PAUaM+iEOVhi24mkBGZWd+KP5u0wr9I
+1OZMHSp/aGjm12mBsHON7i7WTJgrYDlE/9R1ouY1eQ/rKV1zh5FZiEA2OmIKKbn
G16u2TsMdUka8lKeC9gVviW79SPqWxOf0uJHMmzRQUFpUnWg/X0hsFv/Es3xfm7N
h7qfPaPE0aqKVyaK0eo4/MZPISDMhxqTj6jyWyk6LQI7arWRlH/IrZypBFhu+clC
PilGuKi3AgMBAAECggEAUk6VkoxGPi9C6ogo6ZTKXiihAN8Pf8NAdDts4W/Wdbhk
gy1/FP9/mvUm9KkGc7Tg6kw1yuugK5fYBXqOzPXAAIm5nTKLDJ5bbhkSNFD4X29M
BE7DueMWzX1P/CEDphjbObnWjHooGANAkjLIBdl0Ui8vET+XubvYSrzvDynPOQgL
2XVXU8KPi90s6P8ja5gWG9uYqcrmg54vQKGzL1jn0OkVMWuIHBAgjZkahE/yGPUk
olR8Xb9+mHm2WpblbgXb+x1UBNM8TW29qqxxO3AVFZhKO9noMEY0+++VI0BZneIQ
ggdK+67WLfKu/GKVSzDD22DbtQXwvXmCH73NgzRDgQKBgQDdvHGpWaoDpm5wtm4P
Gq6CbaEflhmkRTxAd7HYuAKEDa93OmTb6nq5o2mSNRcG0C0oF7XhsVX8nOUGfDGs
15RtPWUn3ruOnvv/DvxMO+C12dn2N59/j+7xV3NeSR5VbM5j5cccAHNt4+tqEkzN
qdq8GqABSU//1EHJk2n+T5O0HQKBgQDI8eij3tlfhJqGpha0ItvtWtx+REIcCewc
sqzlHO8bGAIrACIzXh8XEHMrzNPiWSlQcU5PVg97xODMqqw3Xd5XHqIKvdHA1jF2
BMWd5tq+jnmfRmRbKpxaiCggGnNsXPtWTFCSNPaEx7YOMKMcyfBbCd4aEdjGiTBF
QamiTOdP4wKBgF1zv5+1V49ERMWiTY52G1iDJeYvF82BFJzDFaSWIRFQx0QDy2BY
WbMFqUfisjq/4FhGbfSaDfhyk6ABFdqX3UmRF6IPIJNCdEiSI0lp7xIVp+Q6mzFj
EzyKM+hn/q3YNsApppoponyNE7nXzqDbVoHy4r7IDDxxU+zGAWUaWtENAoGAHMT9
wNB8IL/Ue+i1oW7IPBBhNzbAnFETW/x84oSk+yUR0mQ/gUk9fEjfpGq7/1EyqBDA
3Hz+1IKYiNT6uSaYWbLKEm2g5VIFXNdMD9JoiRXO9e3XGnJcVl3eGNKQqfgaB/3/
Hx+0F8icGSX/hHSpE++yu9rIRYyIu7Gt/s2x5G8CgYEAmRiAJlYDNNj24issgzQM
iEd5ltjH2t6HfclAXSss+s9lHibnyLX4Hk7GFfuVy03JtpZbWgvpChqvz8ZVJc9F
cJJV6hadv3IjYUL4LGtgC5zq64VqWH259X/qFO48D6oEVIG3n7lHe7vIGLTDSuAS
AvmI8eHfXt3u61nF5v5WvT0=
-----END PRIVATE KEY-----";

    fn make_service_account_json(token_endpoint: &str) -> String {
        serde_json::json!({
            "client_email": "test@test-project.iam.gserviceaccount.com",
            "private_key": TEST_PRIVATE_KEY,
            "token_uri": token_endpoint
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_google_indexing_jwt_generation() {
        let token_server = MockServer::start().await;
        let indexing_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token",
                "token_type": "Bearer",
                "expires_in": 3600
            })))
            .mount(&token_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/urlNotifications:publish"))
            .and(header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "urlNotificationMetadata": {
                    "url": "https://example.com/page"
                }
            })))
            .mount(&indexing_server)
            .await;

        let mut sa_file = NamedTempFile::new().unwrap();
        let token_url = format!("{}/token", token_server.uri());
        write!(sa_file, "{}", make_service_account_json(&token_url)).unwrap();

        let client = GoogleIndexingClient::new(sa_file.path().to_str().unwrap().to_string())
            .with_endpoints(
                token_url,
                format!("{}/urlNotifications:publish", indexing_server.uri()),
            );

        let url = Url::parse("https://example.com/page").unwrap();
        let result = client.submit(&url).await.unwrap();

        assert!(result.success);
        assert_eq!(result.method, "google");
    }

    #[tokio::test]
    async fn test_google_indexing_missing_service_account() {
        let client = GoogleIndexingClient::new("/nonexistent/path/sa.json".to_string());
        let url = Url::parse("https://example.com/page").unwrap();
        let err = client.submit(&url).await.unwrap_err();

        assert!(matches!(err, IndexerError::FileRead { .. }));
    }
}
