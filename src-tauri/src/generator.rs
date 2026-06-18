use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::error::Result;
use crate::models::{AudioFile, ChanceConfig, Project, Station, Trigger};

/// Generate a complete HOI4 radio mod on disk.
///
/// `output_dir` is the mod root directory (e.g. `.../mod/my_mod`). The function
/// clears any previous content, writes the descriptor and launcher files,
/// generates station `.asset` / `.txt` files under `music/`, and emits the
/// localisation YAML under `localisation/simp_chinese/`.
///
/// `audio_store_dir` is the global directory where transcoded OGG files are
/// stored; referenced files are copied into the project's `music/` folder.
pub fn generate_mod(
    project: &Project,
    stations: &[Station],
    audio_files: &[AudioFile],
    output_dir: &Path,
    audio_store_dir: &Path,
) -> Result<()> {
    // 1. Create/clear output_dir (the mod root directory).
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    // 2. Write descriptor.mod.
    write_descriptor_mod(project, output_dir)?;

    // 3. Write launcher .mod file next to the mod folder.
    write_launcher_mod(project, output_dir)?;

    // 4. Create music/ subdirectory and copy referenced OGG files.
    let music_dir = output_dir.join("music");
    fs::create_dir_all(&music_dir)?;

    for audio in audio_files {
        let src = audio_store_dir.join(&audio.ogg_filename);
        let dst = music_dir.join(&audio.ogg_filename);
        if src.exists() {
            fs::copy(&src, &dst)?;
        }
    }

    // Build a lookup map for audio files.
    let audio_map: HashMap<&str, &AudioFile> =
        audio_files.iter().map(|a| (a.id.as_str(), a)).collect();

    // 5. Generate per-station files.
    for station in stations {
        write_station_asset(station, &music_dir, &audio_map)?;
        write_station_txt(station, &music_dir, &audio_map)?;
    }

    // 6. Create localisation/simp_chinese/ subdirectory.
    let loc_dir = output_dir.join("localisation").join("simp_chinese");
    fs::create_dir_all(&loc_dir)?;

    // 7. Generate localisation YAML with all audio file titles.
    write_localisation(project, audio_files, &loc_dir)?;

    Ok(())
}

fn write_descriptor_mod(project: &Project, output_dir: &Path) -> Result<()> {
    let path = output_dir.join("descriptor.mod");
    let mut file = File::create(&path)?;

    writeln!(file, "name=\"{}\"", escape_hoi4(&project.name))?;
    writeln!(file, "version=\"{}\"", escape_hoi4(&project.version))?;
    writeln!(
        file,
        "supported_version=\"{}\"",
        escape_hoi4(&project.supported_version)
    )?;
    write!(file, "tags={{")?;
    for tag in &project.tags {
        write!(file, "\n\t\"{}\"", escape_hoi4(tag))?;
    }
    writeln!(file, "\n}}")?;

    Ok(())
}

fn write_launcher_mod(project: &Project, output_dir: &Path) -> Result<()> {
    let dir_name = output_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| project.id.clone());
    let parent = output_dir.parent().unwrap_or(Path::new("."));
    let path = parent.join(format!("{}.mod", dir_name));
    let mut file = File::create(&path)?;

    writeln!(file, "name=\"{}\"", escape_hoi4(&project.name))?;
    writeln!(file, "path=\"mod/{}\"", escape_hoi4(&dir_name))?;
    writeln!(file, "version=\"{}\"", escape_hoi4(&project.version))?;
    writeln!(
        file,
        "supported_version=\"{}\"",
        escape_hoi4(&project.supported_version)
    )?;

    Ok(())
}

fn write_station_asset(
    station: &Station,
    music_dir: &Path,
    audio_map: &HashMap<&str, &AudioFile>,
) -> Result<()> {
    let path = music_dir.join(format!("{}.asset", station.id));
    let mut file = File::create(&path)?;

    for entry in &station.entries {
        let Some(audio) = audio_map.get(entry.audio_file_id.as_str()) else {
            continue;
        };

        writeln!(file, "music = {{")?;
        writeln!(file, "\tname = \"{}\"", escape_hoi4(&audio.id))?;
        writeln!(file, "\tfile = \"{}\"", escape_hoi4(&audio.ogg_filename))?;
        writeln!(file, "\tvolume = {}", audio.volume)?;
        writeln!(file, "}}")?;
    }

    Ok(())
}

fn write_station_txt(
    station: &Station,
    music_dir: &Path,
    audio_map: &HashMap<&str, &AudioFile>,
) -> Result<()> {
    let path = music_dir.join(format!("{}.txt", station.id));
    let mut file = File::create(&path)?;

    writeln!(file, "music_station = \"{}\"", escape_hoi4(&station.id))?;
    writeln!(file)?;

    for entry in &station.entries {
        if !audio_map.contains_key(entry.audio_file_id.as_str()) {
            continue;
        }

        writeln!(file, "music = {{")?;
        writeln!(file, "\tsong = \"{}\"", escape_hoi4(&entry.audio_file_id))?;
        writeln!(file, "\tchance = {{")?;
        write_chance(&mut file, &entry.chance, 2)?;
        writeln!(file, "\t}}")?;
        writeln!(file, "}}")?;
    }

    Ok(())
}

fn write_chance(file: &mut File, chance: &ChanceConfig, indent_level: usize) -> Result<()> {
    let indent = "\t".repeat(indent_level);
    writeln!(file, "{}factor = {}", indent, chance.factor)?;

    for modifier in &chance.modifiers {
        writeln!(file, "{}modifier = {{", indent)?;
        let inner = "\t".repeat(indent_level + 1);

        if let Some(factor) = modifier.factor {
            writeln!(file, "{}factor = {}", inner, factor)?;
        }
        if let Some(add) = modifier.add {
            writeln!(file, "{}add = {}", inner, add)?;
        }
        if let Some(base) = modifier.base {
            writeln!(file, "{}base = {}", inner, base)?;
        }

        for trigger in &modifier.triggers {
            writeln!(file, "{}{}", inner, format_trigger(trigger))?;
        }

        writeln!(file, "{}}}", indent)?;
    }

    Ok(())
}

fn format_trigger(trigger: &Trigger) -> String {
    match trigger {
        Trigger::HasWar { value } => {
            format!("has_war = {}", if *value { "yes" } else { "no" })
        }
        Trigger::Tag { value } => format!("tag = {}", value),
        Trigger::HasGovernment { ideology } => format!("has_government = {}", ideology),
        Trigger::IsInFaction { tag } => format!("is_in_faction_with = {}", tag),
    }
}

fn write_localisation(
    project: &Project,
    audio_files: &[AudioFile],
    loc_dir: &Path,
) -> Result<()> {
    let path = loc_dir.join(format!("{}_music_l_simp_chinese.yml", project.id));
    let mut file = File::create(&path)?;

    writeln!(file, "l_simp_chinese:")?;
    writeln!(
        file,
        " {}_music_TITLE:0 \"{}\"",
        project.id,
        escape_hoi4(&project.name)
    )?;

    for audio in audio_files {
        writeln!(
            file,
            " {}:0 \"{}\"",
            audio.id,
            escape_hoi4(&audio.title)
        )?;
    }

    Ok(())
}

/// Escape backslashes and double quotes for HOI4 script strings.
fn escape_hoi4(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_hoi4_handles_quotes_and_backslashes() {
        assert_eq!(escape_hoi4(r#"a"b\c"#), r#"a\"b\\c"#);
    }
}
