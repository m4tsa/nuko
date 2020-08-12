pub fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
