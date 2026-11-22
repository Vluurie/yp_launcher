pub fn anchor_for(text: &str) -> String {
    let mut anchor = String::new();
    let mut pending_dash = false;
    for character in text.to_lowercase().chars() {
        let code = u32::from(character);
        let is_word = character.is_ascii_digit()
            || character.is_ascii_lowercase()
            || code > 0x7f
            || character == '_';
        if is_word || character == '-' {
            if pending_dash && !anchor.is_empty() {
                anchor.push('-');
            }
            pending_dash = false;
            anchor.push(character);
        } else if character == ' ' || character == '\t' {
            pending_dash = true;
        }
    }
    anchor
}

pub fn anchor_in(href: &str) -> Option<&str> {
    let (_, fragment) = href.split_once('#')?;
    (!fragment.is_empty()).then_some(fragment)
}
