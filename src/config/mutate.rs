use std::{fs, path::Path};

use termy_config_core::{Rgb8, parse_theme_id};

use super::io::{ensure_config_file, notify_config_changed};

fn update_config_contents<R>(
    updater: impl FnOnce(&str) -> Result<(String, R), String>,
) -> Result<R, String> {
    let config_path =
        ensure_config_file().ok_or_else(|| "Could not locate config file".to_string())?;
    let existing =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;
    let (updated, result) = updater(&existing)?;
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;
    notify_config_changed();
    Ok(result)
}

fn upsert_theme_assignment(contents: &str, theme_id: &str) -> String {
    let mut new_config = String::new();
    let mut replaced = false;
    let mut inserted_before_first_section = false;
    let mut in_root_section = true;

    for line in contents.lines() {
        let trimmed = line.trim();
        let is_section_header = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_section_header {
            if !replaced && !inserted_before_first_section {
                new_config.push_str(&format!("theme = {}\n", theme_id));
                inserted_before_first_section = true;
            }
            in_root_section = false;
            new_config.push_str(line);
            new_config.push('\n');
            continue;
        }

        if in_root_section {
            let mut parts = trimmed.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            if key.eq_ignore_ascii_case("theme") {
                if !replaced {
                    new_config.push_str(&format!("theme = {}\n", theme_id));
                    replaced = true;
                }
                continue;
            }
        }

        new_config.push_str(line);
        new_config.push('\n');
    }

    if !replaced && !inserted_before_first_section {
        if !new_config.is_empty() && !new_config.ends_with('\n') {
            new_config.push('\n');
        }
        new_config.push_str(&format!("theme = {}\n", theme_id));
    }

    new_config
}

fn replace_or_insert_section(contents: &str, section_name: &str, section_lines: &[String]) -> String {
    let mut new_config = String::new();
    let mut in_target_section = false;
    let mut target_section_found = false;
    let target_header = format!("[{}]", section_name);

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_target_section = false;
            if trimmed.eq_ignore_ascii_case(&target_header) {
                target_section_found = true;
                in_target_section = true;
                new_config.push_str(line);
                new_config.push('\n');
                for section_line in section_lines {
                    new_config.push_str(section_line);
                    new_config.push('\n');
                }
                continue;
            }
        }

        if in_target_section {
            continue;
        }

        new_config.push_str(line);
        new_config.push('\n');
    }

    if !target_section_found {
        if !new_config.is_empty() {
            new_config.push('\n');
        }
        new_config.push_str(&target_header);
        new_config.push('\n');
        for section_line in section_lines {
            new_config.push_str(section_line);
            new_config.push('\n');
        }
    }

    new_config
}

pub fn import_colors_from_json(json_path: &Path) -> Result<String, String> {
    let contents =
        fs::read_to_string(json_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid JSON: {}", e))?;

    let colors = json
        .as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    let mut color_lines = Vec::new();

    for (key, value) in colors {
        if key.starts_with('$') {
            continue;
        }

        let hex = value
            .as_str()
            .ok_or_else(|| format!("Color '{}' must be a hex string", key))?;

        if Rgb8::from_hex(hex).is_none() {
            return Err(format!("Invalid hex color for '{}': {}", key, hex));
        }

        let config_key = match key.to_ascii_lowercase().as_str() {
            "foreground" | "fg" => "foreground",
            "background" | "bg" => "background",
            "cursor" => "cursor",
            "black" | "color0" => "black",
            "red" | "color1" => "red",
            "green" | "color2" => "green",
            "yellow" | "color3" => "yellow",
            "blue" | "color4" => "blue",
            "magenta" | "color5" => "magenta",
            "cyan" | "color6" => "cyan",
            "white" | "color7" => "white",
            "bright_black" | "brightblack" | "color8" => "bright_black",
            "bright_red" | "brightred" | "color9" => "bright_red",
            "bright_green" | "brightgreen" | "color10" => "bright_green",
            "bright_yellow" | "brightyellow" | "color11" => "bright_yellow",
            "bright_blue" | "brightblue" | "color12" => "bright_blue",
            "bright_magenta" | "brightmagenta" | "color13" => "bright_magenta",
            "bright_cyan" | "brightcyan" | "color14" => "bright_cyan",
            "bright_white" | "brightwhite" | "color15" => "bright_white",
            _ => continue,
        };

        color_lines.push(format!("{} = {}", config_key, hex));
    }

    if color_lines.is_empty() {
        return Err("No valid colors found in JSON".to_string());
    }

    let color_count = color_lines.len();
    update_config_contents(|existing| {
        Ok((
            replace_or_insert_section(existing, "colors", &color_lines),
            (),
        ))
    })?;
    Ok(format!("Imported {} colors", color_count))
}

pub fn set_theme_in_config(theme_id: &str) -> Result<String, String> {
    let theme = parse_theme_id(theme_id).ok_or_else(|| "Invalid theme id".to_string())?;
    update_config_contents(|existing| {
        Ok((
            upsert_theme_assignment(existing, &theme),
            format!("Theme set to {}", theme),
        ))
    })
}

fn upsert_config_value(contents: &str, key: &str, value: &str) -> String {
    let mut new_config = String::new();
    let mut replaced = false;
    let mut in_root_section = true;

    for line in contents.lines() {
        let trimmed = line.trim();
        let is_section_header = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_section_header {
            if !replaced && in_root_section {
                new_config.push_str(&format!("{} = {}\n", key, value));
                replaced = true;
            }
            in_root_section = false;
        }

        if in_root_section && !trimmed.starts_with('#') {
            let mut parts = trimmed.splitn(2, '=');
            let line_key = parts.next().unwrap_or("").trim();
            if line_key.eq_ignore_ascii_case(key) {
                if !replaced {
                    new_config.push_str(&format!("{} = {}\n", key, value));
                    replaced = true;
                }
                continue;
            }
        }

        new_config.push_str(line);
        new_config.push('\n');
    }

    if !replaced {
        if !new_config.is_empty() && !new_config.ends_with('\n') {
            new_config.push('\n');
        }
        new_config.push_str(&format!("{} = {}\n", key, value));
    }

    new_config
}

pub fn set_config_value(key: &str, value: &str) -> Result<(), String> {
    update_config_contents(|existing| Ok((upsert_config_value(existing, key, value), ())))
}

#[cfg(test)]
mod tests {
    use super::{replace_or_insert_section, upsert_theme_assignment};

    #[test]
    fn upsert_theme_assignment_replaces_existing_root_theme() {
        let input = "theme = termy\nfont_size = 14\n";
        let output = upsert_theme_assignment(input, "nord");
        assert_eq!(output, "theme = nord\nfont_size = 14\n");
    }

    #[test]
    fn upsert_theme_assignment_inserts_before_first_section_when_missing() {
        let input = "font_size = 14\n\n[colors]\nforeground = #ffffff\n";
        let output = upsert_theme_assignment(input, "tokyo-night");
        assert_eq!(
            output,
            "font_size = 14\n\ntheme = tokyo-night\n[colors]\nforeground = #ffffff\n"
        );
    }

    #[test]
    fn replace_or_insert_section_replaces_existing_section_body() {
        let input = "theme = termy\n[colors]\nforeground = #ffffff\nbackground = #000000\n";
        let output = replace_or_insert_section(
            input,
            "colors",
            &[
                "foreground = #111111".to_string(),
                "cursor = #222222".to_string(),
            ],
        );

        assert_eq!(
            output,
            "theme = termy\n[colors]\nforeground = #111111\ncursor = #222222\n"
        );
    }

    #[test]
    fn replace_or_insert_section_appends_missing_section() {
        let input = "theme = termy\nfont_size = 14\n";
        let output =
            replace_or_insert_section(input, "colors", &["foreground = #111111".to_string()]);

        assert_eq!(
            output,
            "theme = termy\nfont_size = 14\n\n[colors]\nforeground = #111111\n"
        );
    }
}
