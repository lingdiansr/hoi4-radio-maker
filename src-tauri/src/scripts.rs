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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::models::WorkshopMod;

/// A trigger documented by the game, with the scopes it may be used in.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TriggerDef {
    pub name: String,
    /// Supported scopes as documented, e.g. `["COUNTRY"]`. May be empty when
    /// the documentation omits the line.
    pub scopes: Vec<String>,
    /// Supported targets as documented, e.g. `["THIS", "ROOT", "PREV"]`, or
    /// `["none"]` / `["any"]`. Scope keywords here are valid values, so they
    /// double as value suggestions in the editor.
    pub targets: Vec<String>,
}

/// The kind of value a trigger expects, inferred from how the game's own
/// scripts use it (the documentation does not state value types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueKind {
    Boolean,
    Number,
    Text,
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
                let scopes = split_list(rest);
                out.push(TriggerDef {
                    name,
                    scopes,
                    targets: Vec::new(),
                });
            }
        } else if let Some(rest) = trimmed.strip_prefix("* Supported Targets:") {
            // Attach to the definition the preceding `Supported Scopes` line opened.
            if let Some(def) = out.last_mut() {
                if def.targets.is_empty() {
                    def.targets = split_list(rest);
                }
            }
        }
    }

    out
}

/// Drop a trailing `#` comment from a script line.
///
/// Script authors routinely annotate definitions (`totalitarian_socialist = { #社`),
/// so comments must be removed before structural checks such as "ends with `{`"
/// or brace counting. `#` inside a double-quoted string is kept.
fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Split a comma-separated documentation list into trimmed, non-empty items.
fn split_list(rest: &str) -> Vec<String> {
    rest.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Collect top-level `<name> = {` blocks from a `scripted_triggers` file.
///
/// Only unindented assignments are definitions; nested `key = value` lines are
/// the trigger's body.
pub fn parse_scripted_triggers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = strip_comment(raw);
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

    for raw in text.lines() {
        // Strip trailing comments first: they contain braces-free prose but
        // would break the `ends_with('{')` check and the brace counting.
        let line = strip_comment(raw);
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
                targets: Vec::new(),
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
pub fn build_vocabulary(roots: &[PathBuf]) -> ScriptVocabulary {
    let mut vocab = ScriptVocabulary::default();
    for root in roots {
        vocab.absorb(vocabulary_from_root(root));
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

/// Directories under a root that contain script using triggers.
const SCRIPT_DIRS: &[&str] = &["common", "events", "history", "decisions"];

/// Upper bound on files inspected by a value-kind lookup, so a pathological
/// install cannot stall the UI.
const MAX_SCAN_FILES: usize = 12_000;

/// Observations needed before a lookup concludes a trigger's value kind.
const KIND_SAMPLE: u32 = 5;

/// Collect `*.txt` files under the trigger-bearing script directories of a
/// root, in a deterministic (sorted) order.
fn collect_script_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in SCRIPT_DIRS {
        collect_txt(&root.join(dir), &mut files);
        if files.len() >= MAX_SCAN_FILES {
            break;
        }
    }
    files.truncate(MAX_SCAN_FILES);
    // Sorted so an early-exit lookup is deterministic across runs.
    files.sort();
    files
}

fn collect_txt(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_SCAN_FILES {
            return;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_txt(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            out.push(path);
        }
    }
}

fn trim_bytes(mut s: &[u8]) -> &[u8] {
    while let Some((first, rest)) = s.split_first() {
        if first.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    while let Some((last, rest)) = s.split_last() {
        if last.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    s
}

/// Classify a script value token: `yes`/`no`, a number, or free text.
/// Returns `None` for tokens that are not a single value (blocks, lists).
fn classify_value(value: &[u8]) -> Option<ValueKind> {
    if value == b"yes" || value == b"no" {
        return Some(ValueKind::Boolean);
    }
    if value.is_empty() {
        return None;
    }
    if value
        .iter()
        .any(|b| b.is_ascii_whitespace() || matches!(b, b'{' | b'}' | b'#'))
    {
        return None;
    }
    let mut digit = false;
    for (i, b) in value.iter().enumerate() {
        if b.is_ascii_digit() {
            digit = true;
        } else if matches!(b, b'-' | b'+' | b'.')
            && (i == 0 || value[i - 1].is_ascii_digit())
        {
            // sign or decimal point in a numeric token
        } else {
            return Some(ValueKind::Text);
        }
    }
    if digit {
        Some(ValueKind::Number)
    } else {
        None
    }
}

/// Read one script line as an assignment, yielding its key and the kind of its
/// value. Returns `None` for anything that is not a single-valued assignment:
/// blank lines, nested blocks, comments, and keys that cannot be triggers.
fn parse_assignment(raw: &[u8]) -> Option<(&[u8], ValueKind)> {
    let line = trim_bytes(raw);
    // Values are often followed by a comment (`= yes # note`), which would
    // otherwise make the token look like free text.
    let line = match line.iter().position(|b| *b == b'#') {
        Some(i) => trim_bytes(&line[..i]),
        None => line,
    };
    // Cheap pre-filter: script keys start with a lowercase letter.
    let (first, _) = line.split_first()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let eq = line.iter().position(|b| *b == b'=')?;
    let key = trim_bytes(&line[..eq]);
    if key.is_empty() {
        return None;
    }
    let mut value = trim_bytes(&line[eq + 1..]);
    if value.len() >= 2 && value[0] == b'"' && value[value.len() - 1] == b'"' {
        value = &value[1..value.len() - 1];
    }
    Some((key, classify_value(value)?))
}

/// The value kind of every trigger the loaded scripts assign, built in one pass
/// over the game and mod scripts.
///
/// Answering "what does this trigger take?" by scanning for one name at a time
/// costs a full read of the corpus per name — 120 MB across 10k files, measured
/// at 2.9s in a debug build, and paid again for the next name. Building the map
/// once costs a single pass and then answers every name from memory.
///
/// Keys are gathered as bytes: they are ASCII identifiers, so this avoids a
/// UTF-8 check and an allocation per line.
pub struct ValueKindIndex {
    kinds: HashMap<Vec<u8>, ValueKind>,
}

impl ValueKindIndex {
    /// Scan every script file below `roots` and classify each assignment.
    pub fn build(roots: &[PathBuf]) -> Self {
        let mut samples: HashMap<Vec<u8>, KindSamples> = HashMap::new();

        for root in roots {
            for file in collect_script_files(root) {
                let Ok(data) = std::fs::read(&file) else {
                    continue;
                };
                for raw in data.split(|b| *b == b'\n') {
                    let Some((key, kind)) = parse_assignment(raw) else {
                        continue;
                    };
                    match samples.get_mut(key) {
                        Some(entry) => entry.push(kind),
                        None => {
                            let mut entry = KindSamples::default();
                            entry.push(kind);
                            samples.insert(key.to_vec(), entry);
                        }
                    }
                }
            }
        }

        let kinds = samples
            .into_iter()
            .filter_map(|(key, entry)| entry.conclude().map(|kind| (key, kind)))
            .collect();

        Self { kinds }
    }

    /// The kind of `name`, or `None` when the scripts only ever write it as a
    /// block (or never mention it at all).
    pub fn get(&self, name: &str) -> Option<ValueKind> {
        self.kinds.get(name.as_bytes()).copied()
    }

    /// Number of triggers with a known kind.
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }
}

/// Observations gathered for one key, capped at [`KIND_SAMPLE`].
#[derive(Default)]
struct KindSamples {
    counts: [u32; 3],
    seen: u32,
}

impl KindSamples {
    fn push(&mut self, kind: ValueKind) {
        if self.seen >= KIND_SAMPLE {
            return;
        }
        self.counts[kind as usize] += 1;
        self.seen += 1;
    }

    /// Majority wins; ties favour the more permissive text kind.
    fn conclude(&self) -> Option<ValueKind> {
        if self.seen == 0 {
            return None;
        }
        let mut best = 0usize;
        for (i, count) in self.counts.iter().enumerate() {
            if *count > self.counts[best] || (*count == self.counts[best] && i > best && *count > 0) {
                best = i;
            }
        }
        match best {
            0 => Some(ValueKind::Boolean),
            1 => Some(ValueKind::Number),
            _ => Some(ValueKind::Text),
        }
    }
}

pub fn script_roots(load_vanilla: bool, game_dir: Option<&Path>, mod_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if load_vanilla {
        if let Some(g) = game_dir {
            roots.push(g.to_path_buf());
        }
    }
    roots.extend(mod_dirs.iter().cloned());
    roots
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
* Supported Targets: THIS, ROOT, PREV

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
    fn parses_supported_targets_onto_the_definition() {
        let defs = parse_trigger_documentation(DOC);
        let has_war = defs.iter().find(|d| d.name == "has_war").unwrap();
        assert_eq!(has_war.scopes, vec!["COUNTRY", "STATE"]);
        assert_eq!(has_war.targets, vec!["THIS", "ROOT", "PREV"]);
        let gov = defs.iter().find(|d| d.name == "has_government").unwrap();
        assert_eq!(gov.targets, vec!["none"]);
    }

    #[test]
    fn classifies_value_tokens() {
        assert_eq!(classify_value(b"yes"), Some(ValueKind::Boolean));
        assert_eq!(classify_value(b"no"), Some(ValueKind::Boolean));
        assert_eq!(classify_value(b"556"), Some(ValueKind::Number));
        assert_eq!(classify_value(b"0.7"), Some(ValueKind::Number));
        assert_eq!(classify_value(b"-12"), Some(ValueKind::Number));
        assert_eq!(classify_value(b"CHI"), Some(ValueKind::Text));
        assert_eq!(classify_value(b"democratic"), Some(ValueKind::Text));
        // Blocks, lists, and empties are not single values.
        assert_eq!(classify_value(b"{"), None);
        assert_eq!(classify_value(b"a b"), None);
        assert_eq!(classify_value(b""), None);
    }

    #[test]
    fn infers_value_kind_from_script_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let common = tmp.path().join("common");
        std::fs::create_dir_all(&common).unwrap();

        let mut body = String::new();
        for _ in 0..6 {
            body.push_str("has_war = yes\nstockpile_ratio = 0.7\noriginal_tag = NOR\n");
        }
        std::fs::write(common.join("usage.txt"), body).unwrap();

        let roots = vec![tmp.path().to_path_buf()];
        let index = ValueKindIndex::build(&roots);
        assert_eq!(index.get("has_war"), Some(ValueKind::Boolean));
        assert_eq!(index.get("stockpile_ratio"), Some(ValueKind::Number));
        assert_eq!(index.get("original_tag"), Some(ValueKind::Text));
        // A trigger the scripts never use has no inferred kind.
        assert_eq!(index.get("never_used_trigger"), None);
        assert_eq!(index.get(""), None);
    }

    #[test]
    fn script_roots_follow_the_project_sources() {
        let game = PathBuf::from("/game");
        let mods = vec![PathBuf::from("/mods/a"), PathBuf::from("/mods/b")];

        assert!(script_roots(false, Some(&game), &[]).is_empty());
        assert_eq!(script_roots(true, Some(&game), &[]), vec![game.clone()]);
        assert_eq!(script_roots(false, Some(&game), &mods), mods);
        assert_eq!(script_roots(true, Some(&game), &mods).len(), 3);
    }

    #[test]
    fn strip_comment_removes_trailing_comments_but_keeps_quoted_hashes() {
        assert_eq!(strip_comment("name = { # note"), "name = { ");
        assert_eq!(strip_comment("name = {"), "name = {");
        assert_eq!(strip_comment("f = \"a#b\" # tail"), "f = \"a#b\" ");
        assert_eq!(strip_comment("# whole line"), "");
    }

    /// Regression: real mod files annotate definitions and use CRLF
    /// (`\ttotalitarian_socialist = { #社\r`), which broke an earlier
    /// `ends_with('{')` check and silently dropped that ideology.
    #[test]
    fn parses_ideologies_with_trailing_comments_and_crlf() {
        let text = "ideologies = {\r\n\r\n\ttotalitarian_socialist = { #社\r\n\t\ttypes = {\r\n\t\t\tjucheism = {\r\n\t\t\t}\r\n\t\t}\r\n\t}\r\n\tcommunist = {\r\n\t}\r\n}\r\n";
        assert_eq!(
            parse_ideologies(text),
            vec!["totalitarian_socialist".to_string(), "communist".to_string()]
        );
    }

    /// Regression: a commented-out definition must not be collected.
    #[test]
    fn commented_out_definitions_are_ignored() {
        let text = "ideologies = {\n\t#legacy = {\n\tcommunist = {\n\t}\n}\n";
        assert_eq!(parse_ideologies(text), vec!["communist".to_string()]);

        let scripted = "#disabled_trigger = {\nreal_trigger = { # note\n\talways = yes\n}\n";
        assert_eq!(
            parse_scripted_triggers(scripted),
            vec!["real_trigger".to_string()]
        );
    }

    /// Regression: a commented value (`= yes # note`) must still be classified.
    #[test]
    fn commented_values_still_contribute_to_value_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let common = tmp.path().join("common");
        std::fs::create_dir_all(&common).unwrap();
        let mut body = String::new();
        for i in 0..6 {
            body.push_str(&format!("has_war = yes # note {i}\r\n"));
        }
        std::fs::write(common.join("usage.txt"), body).unwrap();
        let index = ValueKindIndex::build(&[tmp.path().to_path_buf()]);
        assert_eq!(index.get("has_war"), Some(ValueKind::Boolean));
    }

    #[test]
    fn missing_roots_yield_an_empty_vocabulary() {
        let vocab = build_vocabulary(&[PathBuf::from("/nonexistent/game")]);
        assert!(vocab.is_empty());
    }
}
