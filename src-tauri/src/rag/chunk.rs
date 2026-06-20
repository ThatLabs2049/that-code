const CHUNK_SIZE: usize = 1200;
const CHUNK_OVERLAP: usize = 200;

pub fn chunk_text(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    if trimmed.chars().count() <= CHUNK_SIZE {
        return vec![trimmed.to_string()];
    }

    let mut chunks = Vec::new();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + CHUNK_SIZE).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        if end >= chars.len() {
            break;
        }
        start = end.saturating_sub(CHUNK_OVERLAP);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_long_text() {
        let text = "word ".repeat(400);
        let chunks = chunk_text(&text);
        assert!(chunks.len() > 1);
    }
}
