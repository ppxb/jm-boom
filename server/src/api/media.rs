pub(crate) fn cover_url(comic_id: &str, fallback: &str) -> String {
    if !comic_id.is_empty() && comic_id.chars().all(|character| character.is_ascii_digit()) {
        format!("/api/covers/{comic_id}")
    } else {
        fallback.to_string()
    }
}
