use std::{collections::HashSet, path::Path};

use crate::monitor::MonitorMode;
use dirs::home_dir;
use std::io::Write;

const HYPR_CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
const HYPR_OVERRIDES_PATH: &str = ".config/hypr/conf-overrides.conf";

const MONITOR_CONFIG_PREFIX: &str = "monitor=";
const KEYBOARD_LAYOUT_PREFIX: &str = "input:kb_layout=";

/// Trait for configuration lines that can be overridden in the config file
trait ConfigLine {
    /// Get the prefix that identifies this type of config line
    fn prefix(&self) -> &str;
    
    /// Extract the identifier/key from a config line (e.g., monitor name, setting key)
    /// Returns None if the line doesn't match this config type
    fn extract_key(&self, line: &str) -> Option<String>;
    
    /// Check if this line should replace an existing line with the same key
    fn should_replace(&self) -> bool {
        true
    }
}

struct MonitorConfig;
struct KeyboardLayoutConfig;

impl ConfigLine for MonitorConfig {
    fn prefix(&self) -> &str {
        MONITOR_CONFIG_PREFIX
    }
    
    fn extract_key(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if let Some(config) = trimmed.strip_prefix(self.prefix())
            && let Some(comma_pos) = config.find(',')
        {
            return Some(config[..comma_pos].to_string());
        }
        None
    }
}

impl ConfigLine for KeyboardLayoutConfig {
    fn prefix(&self) -> &str {
        KEYBOARD_LAYOUT_PREFIX
    }
    
    fn extract_key(&self, line: &str) -> Option<String> {
        if line.trim().starts_with(self.prefix()) {
            // For keyboard layout, we use a constant key since there's only one
            Some("kb_layout".to_string())
        } else {
            None
        }
    }
}

/// Registry of all known config line types
fn get_config_handlers() -> Vec<Box<dyn ConfigLine>> {
    vec![
        Box::new(MonitorConfig),
        Box::new(KeyboardLayoutConfig),
    ]
}

/// Create the overrides configuration file for hyprland.
/// This file is created at `~./config/hypr/conf-overrides.conf`
/// If the file already exists, it will not be overwritten.
/// This function will only run once if the file does not exist.
/// It will also edit the main file `~/.config/hypr/hyprland.conf` to include the overrides file as
/// a source file at the bottom of the main configuration file to ensure that all exisiting
/// settings are overwritten but not removed.
pub fn create_overrides() -> anyhow::Result<()> {
    let home_dir = home_dir().ok_or_else(|| {
        anyhow::anyhow!("Could not determine home directory for the current user")
    })?;

    let hypr_config_path = home_dir.join(HYPR_CONFIG_PATH);
    let hypr_overrides_path = home_dir.join(HYPR_OVERRIDES_PATH);

    if !Path::new(&hypr_config_path).exists() {
        return Err(anyhow::anyhow!(
            "Hyprland configuration file not found at {}, Hyprland is either not installed or not configured",
            HYPR_CONFIG_PATH
        ));
    }

    if !Path::new(&hypr_overrides_path).exists() {
        std::fs::write(&hypr_overrides_path, "# Hyprland configuration overrides\n")?;

        // append the file source line to main conf
        let mut hypr_config_file = std::fs::OpenOptions::new()
            .append(true)
            .open(&hypr_config_path)?;

        writeln!(
            hypr_config_file,
            "\n# Include overrides configuration\nsource = ~/{}",
            HYPR_OVERRIDES_PATH
        )?;
    }

    Ok(())
}

/// Write a line to the overrides configuartion file.
/// Uses dynamic config handlers to determine if the line should replace an existing one.
/// For monitor configs, it will replace lines with the same monitor name.
/// For keyboard layout, it will replace the existing layout line.
pub fn write_override_line(line: &str) -> anyhow::Result<()> {
    let home_dir = home_dir().ok_or_else(|| {
        anyhow::anyhow!("Could not determine home directory for the current user")
    })?;

    let hypr_overrides_path = home_dir.join(HYPR_OVERRIDES_PATH);

    // Read existing content
    let content = std::fs::read_to_string(&hypr_overrides_path)?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let handlers = get_config_handlers();
    let mut replaced = false;

    // Try to find a handler that matches this line
    for handler in &handlers {
        if let Some(new_key) = handler.extract_key(line) {
            if handler.should_replace() {
                // Find and replace existing line with the same key
                for existing_line in lines.iter_mut() {
                    if let Some(existing_key) = handler.extract_key(existing_line)
                        && existing_key == new_key
                    {
                        *existing_line = line.to_string();
                        replaced = true;
                        break;
                    }
                }
            }
            break;
        }
    }

    // If not replaced, append the new line
    if !replaced {
        lines.push(line.to_string());
    }

    // Write back to file
    let updated_content = lines.join("\n") + "\n";
    std::fs::write(&hypr_overrides_path, updated_content)?;

    Ok(())
}


/// Generate a monitor override string for hyprland configuration.
/// Currently just generates the basic one as i am a europoor and only have 1 monitor to test with.
pub fn monitor_override(monitor_name: String, settings: MonitorMode) -> String {
    // for now we want this, as I only have 1 monitor to test with so position settings are TODO
    let auto_position_string = format!("{}@{},auto,1", settings.resolution, settings.refresh_rate);

    format!(
        "{}{},{}",
        MONITOR_CONFIG_PREFIX, monitor_name, auto_position_string
    )
}

/// Generate a keyboard locale override string for hyprland configuration.
/// E.g., for locales {"us", "fi"}, generates "input:kb_layout=us,fi"
pub fn locale_override(locale: HashSet<String>) -> String {
    let mut input_locale_setting = String::from("input:kb_layout=");

    let locales: Vec<&str> = locale.iter().map(|s| s.as_str()).collect();
    input_locale_setting.push_str(&locales.join(","));

    input_locale_setting
}

// Capitalise the locales so they are easier to read for now
// TODO: Find some form of default list of 2-letter locale codes with proper names
// e.g., "us" -> "US", "fi" -> "FI". Maybe hyprland config has a list
#[allow(dead_code)]
pub fn humanize_locale(locale: &str) -> String {
    locale
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_extract_key() {
        let monitor_config = MonitorConfig;
        assert_eq!(
            monitor_config.extract_key("monitor=DP-3,2560x1440@155,auto,1"),
            Some("DP-3".to_string())
        );
        assert_eq!(
            monitor_config.extract_key("  monitor=HDMI-A-1,1920x1080@60,auto,1  "),
            Some("HDMI-A-1".to_string())
        );
        assert_eq!(monitor_config.extract_key("# comment line"), None);
        assert_eq!(monitor_config.extract_key("some other config"), None);
    }

    #[test]
    fn test_keyboard_config_extract_key() {
        let kb_config = KeyboardLayoutConfig;
        assert_eq!(
            kb_config.extract_key("input:kb_layout=us,fi"),
            Some("kb_layout".to_string())
        );
        assert_eq!(kb_config.extract_key("monitor=DP-3,auto"), None);
    }
}
