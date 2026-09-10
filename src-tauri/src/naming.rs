//! Naming helpers shared by the id generators and the mod writer.

/// Convert a human-readable name into a HOI4-safe ASCII slug: lowercase
/// alphanumerics and underscores only. Returns an empty string when the input
/// has no ASCII alphanumeric characters (e.g. pure CJK).
///
/// Every other character — including path separators, dots, and non-ASCII
/// letters — collapses to `_`, so a slug can never escape its parent directory.
pub fn slugify_id(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    // collapse consecutive underscores and trim edges
    out.split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Append `_2`, `_3`, … until `base` is free in `used`, then claim it.
///
/// Shared by station dir resolution so two stations can never target the same
/// folder. Bounded to keep a pathological name from looping forever.
pub fn unique_name(base: &str, used: &mut std::collections::HashSet<String>) -> String {
    let mut candidate = base.to_string();
    let mut n = 2u32;
    while used.contains(&candidate) {
        if n > 10_000 {
            break;
        }
        candidate = format!("{base}_{n}");
        n += 1;
    }
    used.insert(candidate.clone());
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_underscores() {
        assert_eq!(slugify_id("My Song Title"), "my_song_title");
        assert_eq!(slugify_id("Song - 01 (Remix)"), "song_01_remix");
        assert_eq!(slugify_id("Already_slug"), "already_slug");
    }

    #[test]
    fn slugify_returns_empty_for_pure_cjk_or_whitespace() {
        assert_eq!(slugify_id("东方红"), "");
        assert_eq!(slugify_id("  "), "");
    }

    #[test]
    fn slugify_neutralises_path_characters() {
        assert_eq!(slugify_id("../../etc"), "etc");
        assert_eq!(slugify_id("a/b..c"), "a_b_c");
    }

    #[test]
    fn unique_name_suffixes_on_collision() {
        let mut used = std::collections::HashSet::new();
        assert_eq!(unique_name("radio", &mut used), "radio");
        assert_eq!(unique_name("radio", &mut used), "radio_2");
        assert_eq!(unique_name("radio", &mut used), "radio_3");
        assert_eq!(unique_name("other", &mut used), "other");
    }
}
