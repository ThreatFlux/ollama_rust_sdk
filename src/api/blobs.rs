//! Blobs API implementation

use crate::{
    error::{OllamaError, Result},
    utils::http::HttpClient,
};
use std::sync::Arc;

/// API implementation for blob management
pub struct BlobsApi;

impl BlobsApi {
    /// Check if a blob exists
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if the server returns an error status.
    pub async fn blob_exists(http_client: &Arc<HttpClient>, digest: &str) -> Result<bool> {
        let path = format!("api/blobs/{digest}");
        let response = http_client.head(&path).await?;

        match response.status().as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            status => Err(OllamaError::ServerError {
                status,
                message: "Blob check failed".to_string(),
            }),
        }
    }

    /// Create/upload a blob
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if the server returns an error status.
    pub async fn create_blob(
        http_client: &Arc<HttpClient>,
        digest: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        let path = format!("api/blobs/{digest}");
        let response = http_client
            .put(&path)
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::ClientConfig, utils::http::HttpClient};
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_blob_digest_format() {
        let digest = "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2";
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 71); // "sha256:" + 64 hex characters
    }

    #[tokio::test]
    async fn test_blob_exists_returns_true_when_blob_exists() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2";

        Mock::given(method("HEAD"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::blob_exists(&http_client, digest).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_blob_exists_returns_false_when_blob_not_found() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:nonexistentblob";

        Mock::given(method("HEAD"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::blob_exists(&http_client, digest).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_blob_exists_returns_error_on_server_error() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2";

        Mock::given(method("HEAD"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::blob_exists(&http_client, digest).await;
        assert!(result.is_err());

        if let Err(OllamaError::ServerError { status, message }) = result {
            assert_eq!(status, 500);
            assert_eq!(message, "Blob check failed");
        } else {
            panic!("Expected ServerError");
        }
    }

    #[tokio::test]
    async fn test_create_blob_success() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2";
        let blob_data = b"test blob data".to_vec();

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .and(header("Content-Type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_blob_with_empty_data() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let blob_data = Vec::new(); // Empty data

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .and(header("Content-Type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_blob_with_large_data() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:largedigest";
        let blob_data = vec![42u8; 1024 * 1024]; // 1MB of data

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .and(header("Content-Type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_blob_server_error() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2";
        let blob_data = b"test blob data".to_vec();

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(409).set_body_string("Blob already exists"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_err());

        // The actual HTTP client's behavior may vary, so just check that we get an error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_blob_bad_request() {
        let mock_server = MockServer::start().await;
        let digest = "invalid-digest";
        let blob_data = b"test blob data".to_vec();

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(400).set_body_string("Invalid digest format"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_blob_with_special_characters_in_data() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:specialchars";
        let blob_data = b"test\x00\x01\x02\xff\xfe\xfd blob data".to_vec();

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .and(header("Content-Type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = BlobsApi::create_blob(&http_client, digest, blob_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_blob_exists_with_various_digest_formats() {
        let mock_server = MockServer::start().await;
        let digests = vec![
            "sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2",
            "sha256:a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ];

        for digest in digests {
            Mock::given(method("HEAD"))
                .and(path(format!("/api/blobs/{digest}")))
                .respond_with(ResponseTemplate::new(200))
                .mount(&mock_server)
                .await;

            let config = ClientConfig {
                base_url: mock_server.uri().parse().unwrap(),
                ..ClientConfig::default()
            };
            let http_client = Arc::new(HttpClient::new(config).unwrap());
            let result = BlobsApi::blob_exists(&http_client, digest).await.unwrap();
            assert!(result);
        }
    }
}
