use chrono::DateTime;
use redb::AccessGuard;

pub fn trim(s: &Vec<u8>) -> Vec<u8> {
    // Original from https://stackoverflow.com/a/67358195 just changed to be used on vectors
    let from = match s.iter().position(|c| !c.is_ascii_whitespace()) {
        Some(i) => i,
        None => return s[0..0].into(),
    };
    let to = s.iter().rposition(|c| !c.is_ascii_whitespace()).unwrap();
    s[from..=to].into()
}

// https://stackoverflow.com/a/38461750
pub fn truncate(s: String, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => s[..idx].into(),
    }
}

pub fn format_entry(
    entry: (AccessGuard<i64>, AccessGuard<Vec<u8>>),
    width: usize,
) -> (String, String) {
    (
        DateTime::from_timestamp_millis(entry.0.value())
            .unwrap()
            .format("%c")
            .to_string(),
        truncate(String::from_utf8(trim(&entry.1.value())).unwrap(), width),
    )
}
