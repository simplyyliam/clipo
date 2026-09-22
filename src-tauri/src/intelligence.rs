//! Clipboard Intelligence: how captured content is interpreted *before* it
//! becomes history.
//!
//! Two focused rules only:
//!   * URL detection, so the UI can present a text clip as a link.
//!   * Sensitive-content detection, so credentials never reach storage.
//!
//! Deliberately not a general content classifier.

/// First URL inside `text`, if the text looks like it carries one.
///
/// Accepts `http(s)://` and bare `www.` hosts, which covers what a user copies
/// from a browser or a document without turning this into a link parser.
pub fn detect_url(text: &str) -> Option<String> {
    let candidate = text
        .split(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '"')
        .find(|token| {
            let lower = token.to_lowercase();
            lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.")
        })?;

    // Trailing punctuation is almost always sentence punctuation, not the URL.
    let trimmed = candidate.trim_end_matches(|c| matches!(c, '.' | ',' | ')' | ']' | '}' | ';' | '!' | '?' | '\''));

    if trimmed.len() < 5 || !trimmed.contains('.') {
        return None;
    }

    Some(trimmed.to_string())
}

/// True when the whole clip is a single URL, which is what the `Link` kind means.
pub fn is_link(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.contains(char::is_whitespace) && detect_url(trimmed).is_some()
}

/// Heuristic credential detection. Biased towards protecting the user: a false
/// positive costs one missing history entry, a false negative stores a secret.
pub fn looks_sensitive(text: &str) -> bool {
    let trimmed = text.trim();

    if trimmed.is_empty() || trimmed.len() > 8_000 {
        return false;
    }

    if trimmed.contains("-----BEGIN") && trimmed.contains("PRIVATE KEY") {
        return true;
    }

    if is_jwt(trimmed) {
        return true;
    }

    if has_credential_keyword(trimmed) {
        return true;
    }

    if is_card_number(trimmed) {
        return true;
    }

    false
}

fn is_jwt(text: &str) -> bool {
    if !text.starts_with("eyJ") {
        return false;
    }

    let segments: Vec<&str> = text.split('.').collect();
    segments.len() == 3
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
        })
}

/// Matches `password: hunter2`, `api_key=abc...` and similar assignments.
fn has_credential_keyword(text: &str) -> bool {
    const KEYWORDS: [&str; 8] = [
        "password",
        "passwd",
        "secret",
        "api key",
        "api_key",
        "apikey",
        "access token",
        "access_token",
    ];

    text.lines().take(20).any(|line| {
        let lower = line.to_lowercase();
        let Some(separator) = lower.find([':', '=']) else {
            return false;
        };

        let (label, value) = lower.split_at(separator);
        KEYWORDS.iter().any(|keyword| label.contains(keyword))
            && value.trim_start_matches([':', '=']).trim().len() >= 4
    })
}

/// 13-19 digit sequence passing the Luhn checksum, ignoring spaces and dashes.
fn is_card_number(text: &str) -> bool {
    let digits: Vec<u32> = text
        .chars()
        .filter(|c| !matches!(c, ' ' | '-'))
        .map(|c| c.to_digit(10))
        .collect::<Option<Vec<u32>>>()
        .unwrap_or_default();

    if !(13..=19).contains(&digits.len()) {
        return false;
    }

    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(index, digit)| {
            if index % 2 == 1 {
                let doubled = digit * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                *digit
            }
        })
        .sum();

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_urls_and_link_clips() {
        assert_eq!(
            detect_url("see https://clipo.app/docs for more."),
            Some("https://clipo.app/docs".to_string())
        );
        assert!(is_link("https://clipo.app"));
        assert!(!is_link("https://clipo.app is nice"));
        assert_eq!(detect_url("no links here"), None);
    }

    #[test]
    fn detects_sensitive_content() {
        assert!(looks_sensitive("password: hunter2000"));
        assert!(looks_sensitive("-----BEGIN RSA PRIVATE KEY-----\nabc"));
        assert!(looks_sensitive("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123"));
        assert!(looks_sensitive("4242 4242 4242 4242"));
        assert!(!looks_sensitive("Lorem ipsum dolor sit amet"));
        assert!(!looks_sensitive("1234 5678 9012 3456"));
    }
}
