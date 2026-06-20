use crate::settings::AiSettings;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Embedding provider is not configured")]
    NotConfigured,
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("Unexpected embedding response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

pub fn embeddings_url(base_url: &str) -> String {
    format!("{}/embeddings", crate::ai::client::normalize_base_url(base_url))
}

pub fn embedding_api_key(settings: &AiSettings) -> &str {
    if !settings.embedding_api_key.trim().is_empty() {
        settings.embedding_api_key.trim()
    } else {
        settings.api_key.trim()
    }
}

pub async fn create_embedding(settings: &AiSettings, text: &str) -> Result<Vec<f32>, EmbeddingError> {
    if settings.embedding_model.trim().is_empty() {
        return Err(EmbeddingError::NotConfigured);
    }

    let url = embeddings_url(&settings.embedding_base_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(EmbeddingError::Http)?;

    let mut request = client.post(url).json(&serde_json::json!({
        "model": settings.embedding_model,
        "input": text,
    }));

    request = crate::ai::client::auth_request(request, embedding_api_key(settings));

    let response = request.send().await?;
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let message = serde_json::from_str::<crate::ai::types::ApiErrorResponse>(&body)
            .map(|parsed| parsed.error.message)
            .unwrap_or(body);
        return Err(EmbeddingError::Api {
            status: status.as_u16(),
            message,
        });
    }

    let parsed: EmbeddingResponse = serde_json::from_str(&body).map_err(|err| {
        EmbeddingError::InvalidResponse(format!("{err}; body: {body}"))
    })?;

    parsed
        .data
        .into_iter()
        .next()
        .map(|item| item.embedding)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| EmbeddingError::InvalidResponse("missing embedding vector".into()))
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 0.001);
    }
}
