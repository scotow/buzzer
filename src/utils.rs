pub fn sanitize(str: &str) -> &str {
    str.trim()
}

pub fn sanitize_for_search(str: &str) -> Box<str> {
    str.trim().to_lowercase().into_boxed_str()
}
