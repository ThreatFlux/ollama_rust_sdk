//! Builder for generate requests

use crate::{
    api::generate::GenerateApi,
    error::Result,
    models::{
        common::{KeepAlive, Options, ResponseFormat},
        generation::{GenerateRequest, GenerateResponse},
    },
    streaming::stream::GenerateStream,
    utils::http::HttpClient,
};
use std::sync::Arc;

/// Builder for generate requests
#[derive(Debug, Clone)]
pub struct GenerateBuilder {
    http_client: Arc<HttpClient>,
    request: GenerateRequest,
}

impl GenerateBuilder {
    /// Create a new generate builder
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        Self {
            http_client,
            request: GenerateRequest::default(),
        }
    }

    /// Set the model to use
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.request.model = model.into();
        self
    }

    /// Set the prompt
    pub fn prompt<S: Into<String>>(mut self, prompt: S) -> Self {
        self.request.prompt = prompt.into();
        self
    }

    /// Set the system message
    pub fn system<S: Into<String>>(mut self, system: S) -> Self {
        self.request.system = Some(system.into());
        self
    }

    /// Set the template
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.request.template = Some(template.into());
        self
    }

    /// Set the context for conversation continuity
    pub fn context(mut self, context: Vec<i32>) -> Self {
        self.request.context = Some(context);
        self
    }

    /// Set generation options
    pub fn options(mut self, options: Options) -> Self {
        self.request.options = Some(options);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f64) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.temperature = Some(temperature);
        self.request.options = Some(options);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        // Use saturating conversion to avoid wrap-around
        options.num_predict = Some(max_tokens.try_into().unwrap_or(i32::MAX));
        self.request.options = Some(options);
        self
    }

    /// Set top-k
    pub fn top_k(mut self, top_k: i32) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.top_k = Some(top_k);
        self.request.options = Some(options);
        self
    }

    /// Set top-p
    pub fn top_p(mut self, top_p: f64) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.top_p = Some(top_p);
        self.request.options = Some(options);
        self
    }

    /// Set response format
    pub fn format(mut self, format: ResponseFormat) -> Self {
        self.request.format = Some(format);
        self
    }

    /// Set raw mode
    pub fn raw(mut self, raw: bool) -> Self {
        self.request.raw = Some(raw);
        self
    }

    /// Set keep alive
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.request.keep_alive = Some(keep_alive);
        self
    }

    /// Add images for multimodal models
    pub fn images(mut self, images: Vec<String>) -> Self {
        self.request.images = Some(images);
        self
    }

    /// Send the request (non-streaming)
    pub async fn send(self) -> Result<GenerateResponse> {
        GenerateApi::generate(&self.http_client, self.request).await
    }

    /// Send the request with streaming
    pub async fn stream(self) -> Result<GenerateStream> {
        let stream = GenerateApi::generate_stream(&self.http_client, self.request).await?;
        Ok(GenerateStream::new(Box::pin(stream)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_builder() {
        let config = crate::config::ClientConfig::default();
        let http_client = Arc::new(crate::utils::http::HttpClient::new(config).unwrap());

        let builder = GenerateBuilder::new(http_client)
            .model("test-model")
            .prompt("test prompt")
            .temperature(0.7)
            .max_tokens(100);

        assert_eq!(builder.request.model, "test-model");
        assert_eq!(builder.request.prompt, "test prompt");
        assert!(builder.request.options.is_some());

        let options = builder.request.options.unwrap();
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.num_predict, Some(100));
    }
}
