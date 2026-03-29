use crate::common::error::{AppError, AppResult};

pub const EMBEDDING_MODEL: &str = "google-embedding-2";
const EMBEDDING_SIZE: usize = 8;

pub async fn embed(text: &str) -> AppResult<Vec<f32>> {
    let _model = EMBEDDING_MODEL;

    if text.trim().is_empty() {
        return Err(AppError::BadRequest("content cannot be empty".to_string()));
    }

    let mut values = vec![0.0; EMBEDDING_SIZE];

    for (index, byte) in text.bytes().enumerate() {
        let slot = index % EMBEDDING_SIZE;
        values[slot] += f32::from(byte) / 255.0;
    }

    let magnitude = values.iter().map(|value| value * value).sum::<f32>().sqrt();

    if magnitude > 0.0 {
        for value in &mut values {
            *value /= magnitude;
        }
    }

    Ok(values)
}
