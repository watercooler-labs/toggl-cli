pub fn remove_trailing_newline(value: String) -> String {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str().to_string()
}
