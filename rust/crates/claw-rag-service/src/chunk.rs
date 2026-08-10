//! Split file text into overlapping windows (character-based UTF-8).

#[must_use]
pub fn chunk_text(text: &str, max_chars: usize, overlap: usize) -> Vec<String> {
    if max_chars == 0 {
        return Vec::new();
    }
    let overlap = overlap.min(max_chars.saturating_sub(1));
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return out;
    }
    let mut start = 0;
    loop {
        let end = (start + max_chars).min(chars.len());
        let piece: String = chars[start..end].iter().collect();
        if !piece.trim().is_empty() {
            out.push(piece);
        }
        if end >= chars.len() {
            break;
        }
        let step = max_chars.saturating_sub(overlap).max(1);
        start += step;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_non_empty() {
        let c = chunk_text("hello world test", 5, 2);
        assert!(!c.is_empty());
        let joined: String = c.join("");
        assert!(joined.contains("hello"));
    }

    #[test]
    fn short_document_still_gets_one_chunk() {
        for text in [
            "short blade crack note",
            "叶片疑似裂纹，建议复检。",
            "齿轮箱油温升高，需要检查油样和振动。",
        ] {
            let chunks = chunk_text(text, 900, 120);
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0], text);
        }
    }
}
