//! Trimming module.

///Trim whitespaces on each line.
///
///Returns something, if String has whitespaces.
pub fn lines(text: &String) -> Option<String> {
    let orig_len = text.len();
    let last_char = text.chars().last().unwrap();
    let mut text = text.lines().fold(String::with_capacity(orig_len), |acc, line| acc + line.trim_right() + "\n");

    if last_char != '\n' {
        text.pop();
    }

    if orig_len == text.len() {
        None
    }
    else {
        Some(text)
    }
}
