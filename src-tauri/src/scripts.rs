//! Game/mod script vocabulary.
//!
//! The validator needs to know which trigger names, country tags, and
//! ideologies actually exist. Rather than shipping a hardcoded list, the
//! vocabulary is read from the game the user already has installed:
//!
//! * `documentation/triggers_documentation.md` — every built-in trigger with
//!   its supported scopes (vanilla only; mods rarely ship this file).
//! * `common/scripted_triggers/*.txt` — scripted triggers defined by a mod
//!   (or by vanilla), used as additional trigger names.
//! * `common/country_tags/*.txt` and `common/ideologies/*.txt` — the values
//!   `tag` / `is_in_faction_with` / `has_government` take.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::models::WorkshopMod;

/// A trigger documented by the game, with the scopes it may be used in.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TriggerDef {
    pub name: String,
    /// Supported scopes as documented, e.g. `["COUNTRY"]`. May be empty when
    /// the documentation omits the line.
    pub scopes: Vec<String>,
}

/// Trigger names, country tags, and ideologies available to a project.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ScriptVocabulary {
    pub triggers: BTreeMap<String, TriggerDef>,
    pub country_tags: BTreeSet<String>,
    pub ideologies: BTreeSet<String>,
}

impl ScriptVocabulary {
    pub fn is_empty(&self) -> bool {
        self.triggers.is_empty() && self.country_tags.is_empty() && self.ideologies.is_empty()
    }

    /// Merge `other` into `self`; entries already present are kept as-is.
    fn absorb(&mut self, other: ScriptVocabulary) {
        for (name, def) in other.triggers {
            self.triggers.entry(name).or_insert(def);
        }
        self.country_tags.extend(other.country_tags);
        self.ideologies.extend(other.ideologies);
    }
}

/// Parse `documentation/triggers_documentation.md`.
///
/// Definitions look like:
/// ```text
/// ## has_government
///
/// * Supported Scopes: COUNTRY
/// * Supported Targets: none
/// ```
/// Section headings such as `## Triggers for scope COUNTRY` never carry a
/// `Supported Scopes` line, so pairing the two lines picks out the real
/// definitions.
pub fn parse_trigger_documentation(text: &str) -> Vec<TriggerDef> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim_end();
        if let Some(name) = trimmed.strip_prefix("## ") {
            pending = Some(name.trim().to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("* Supported Scopes:") {
            if let Some(name) = pending.take() {
                let scopes = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                out.push(TriggerDef { name, scopes });
            }
        }
    }

    out
}

/// Collect top-level `<name> = {` blocks from a `scripted_triggers` file.
///
/// Only unindented assignments are definitions; nested `key = value` lines are
/// the trigger's body.
pub fn parse_scripted_triggers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        // A definition starts at column 0; anything indented is content.
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some(eq) = line.find('=') else { continue };
        if !line[eq + 1..].trim_start().starts_with('{') {
            continue;
        }
        let name = line[..eq].trim();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.push(name.to_string());
        }
    }
    out
}

/// Collect country tags from `common/country_tags/*.txt` (`GER = "countries/..."`).
pub fn parse_country_tags(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(eq) = trimmed.find('=') else {
            continue;
        };
        let tag = trimmed[..eq].trim();
        if tag.len() == 3 && tag.chars().all(|c| c.is_ascii_uppercase()) {
            out.push(tag.to_string());
        }
    }
    out
}

/// Collect ideology ids from `common/ideologies/*.txt`.
///
/// The file wraps everything in an `ideologies = { ... }` block; the ids are
/// its direct children (`\tdemocratic = {`). Nested blocks such as `types` sit
/// one level deeper and must not be picked up.
pub fn parse_ideologies(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    // Depth relative to the `ideologies` block: 0 = outside, 1 = inside.
    let mut depth = 0i32;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if depth == 0 {
            if trimmed.starts_with("ideologies") && trimmed.contains('{') {
                depth = 1;
            }
            continue;
        }

        // A direct child assignment sits at exactly one indent level.
        let indent = line.len() - line.trim_start().len();
        if depth == 1 && indent == 1 && trimmed.ends_with('{') {
            if let Some(eq) = trimmed.find('=') {
                let name = trimmed[..eq].trim();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    out.push(name.to_string());
                }
            }
        }

        let opens = trimmed.matches('{').count() as i32;
        let closes = trimmed.matches('}').count() as i32;
        depth = (depth + opens - closes).max(0);
    }

    out
}

/// Read every `*.txt` in a directory, ignoring a missing directory.
fn read_dir_texts(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            out.push(text);
        }
    }
    out
}

/// Read the trigger documentation shipped by a root, if any.
fn read_trigger_documentation(root: &Path) -> Vec<TriggerDef> {
    let path = root.join("documentation").join("triggers_documentation.md");
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_trigger_documentation(&text),
        Err(_) => Vec::new(),
    }
}

/// Harvest the vocabulary a single root (game dir or mod dir) contributes.
fn vocabulary_from_root(root: &Path) -> ScriptVocabulary {
    let mut vocab = ScriptVocabulary::default();

    for def in read_trigger_documentation(root) {
        vocab.triggers.insert(def.name.clone(), def);
    }

    for text in read_dir_texts(&root.join("common").join("scripted_triggers")) {
        for name in parse_scripted_triggers(&text) {
            vocab.triggers.entry(name.clone()).or_insert(TriggerDef {
                name,
                scopes: Vec::new(),
            });
        }
    }

    for text in read_dir_texts(&root.join("common").join("country_tags")) {
        vocab.country_tags.extend(parse_country_tags(&text));
    }

    for text in read_dir_texts(&root.join("common").join("ideologies")) {
        vocab.ideologies.extend(parse_ideologies(&text));
    }

    vocab
}

/// Build the vocabulary for a project.
///
/// `load_vanilla` pulls in the installed game's documentation; `mod_dirs` adds
/// each selected mod on top. Everything is best-effort: a missing game
/// directory or unreadable mod yields whatever could be read, never an error.
pub fn build_vocabulary(
    load_vanilla: bool,
    game_dir: Option<&Path>,
    mod_dirs: &[PathBuf],
) -> ScriptVocabulary {
    let mut vocab = ScriptVocabulary::default();

    if load_vanilla {
        if let Some(root) = game_dir {
            vocab.absorb(vocabulary_from_root(root));
        }
    }
    for dir in mod_dirs {
        vocab.absorb(vocabulary_from_root(dir));
    }

    vocab
}

/// The HOI4 Steam application id, used to locate workshop content.
const HOI4_APP_ID: &str = "394360";

/// Locate `<steamlib>/workshop/content/394360` and list the mods inside it.
///
/// The game directory is `<steamlib>/common/Hearts of Iron IV`, so the Steam
/// library is two levels up. Returns an empty list when the layout does not
/// match (non-Steam installs, custom libraries).
pub fn find_workshop_mods(game_dir: &Path) -> Vec<WorkshopMod> {
    let Some(steam_lib) = game_dir.parent().and_then(|p| p.parent()) else {
        return Vec::new();
    };
    let content = steam_lib.join("workshop").join("content").join(HOI4_APP_ID);

    let Ok(entries) = std::fs::read_dir(&content) else {
        return Vec::new();
    };

    let mut mods: Vec<WorkshopMod> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|entry| {
            let path = entry.path();
            let id = entry.file_name().to_string_lossy().to_string();
            let name = read_descriptor_name(&path).unwrap_or_else(|| id.clone());
            WorkshopMod {
                id,
                name,
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect();

    mods.sort_by_key(|m| m.name.to_lowercase());
    mods
}

/// Read `name="..."` from a mod's `descriptor.mod`.
fn read_descriptor_name(mod_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(mod_dir.join("descriptor.mod")).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("name") else {
            continue;
        };
        let rest = rest.trim_start().strip_prefix('=')?.trim();
        let value = rest.trim_matches('"').trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "\
# Triggers

## Table of Content

* [has_war](#has_war)

## Triggers for scope COUNTRY

* [has_war](#has_war)

## has_government

* Supported Scopes: COUNTRY
* Supported Targets: none

## has_war

* Supported Scopes: COUNTRY, STATE

## not_a_trigger

Some prose without a scope line.
";

    #[test]
    fn parses_only_scoped_definitions() {
        let defs = parse_trigger_documentation(DOC);
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["has_government", "has_war"]);
        assert_eq!(defs[0].scopes, vec!["COUNTRY"]);
        assert_eq!(defs[1].scopes, vec!["COUNTRY", "STATE"]);
    }

    #[test]
    fn parses_top_level_scripted_triggers_only() {
        let text = "\
# comment
should_activate = {
\talways = no
\tnested = {
\t\talways = yes
\t}
}

not_a_definition = yes
";
        assert_eq!(
            parse_scripted_triggers(text),
            vec!["should_activate".to_string()]
        );
    }

    #[test]
    fn parses_country_tags() {
        let text = "GER\t= \"countries/Germany.txt\"\n# ENG = commented out\nSOV = \"countries/Soviet Union.txt\"\nnotatag = \"x\"\n";
        assert_eq!(
            parse_country_tags(text),
            vec!["GER".to_string(), "SOV".to_string()]
        );
    }

    #[test]
    fn parses_ideologies() {
        let text = "\
ideologies = {

\tdemocratic = {

\t\ttypes = {
\t\t}
\t}
\tcommunism = {
\t}
}
";
        assert_eq!(
            parse_ideologies(text),
            vec!["democratic".to_string(), "communism".to_string()]
        );
    }

    #[test]
    fn missing_roots_yield_an_empty_vocabulary() {
        let vocab = build_vocabulary(true, Some(Path::new("/nonexistent/game")), &[]);
        assert!(vocab.is_empty());
    }
}
