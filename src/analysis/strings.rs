use std::borrow::Cow;

pub fn decode_string(raw: &str) -> Cow<str> {
    assert!(raw.len() >= 2);
    assert!(raw.as_bytes()[0] == b'"' && raw.as_bytes()[raw.len() - 1] == b'"');
    // drop the surrounding quotes
    let raw = &raw[1..(raw.len() - 1)];
    let escape_count = raw.as_bytes().iter().filter(|&&b| b == b'\\').count();
    if escape_count > 0 {
        let mut decoded = String::with_capacity(raw.len() - escape_count);
        let mut last_was_esc = false;
        for raw in raw.chars() {
            if last_was_esc || raw != '\\' {
                decoded.push(raw);
                last_was_esc = false;
            } else {
                last_was_esc = true;
            }
        }
        Cow::Owned(decoded)
    } else {
        Cow::Borrowed(raw)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
