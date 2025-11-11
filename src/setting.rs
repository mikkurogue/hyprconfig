use std::path::Path;

use crate::setting_writer::{HYPR_OVERRIDES_PATH, HYPR_SETTING_PATH, SettingLine};
use dirs::home_dir;
use std::io::Write;

const MONITOR_SETTING_PREFIX: &str = "monitor=";
const KEYBOARD_LAYOUT_PREFIX: &str = "input:kb_layout=";
const MOUSE_SENSITIVITY_PREFIX: &str = "input:sensitivity=";
const MOUSE_FORCE_NO_ACCEL_PREFIX: &str = "input:force_no_accel=";

struct MonitorSetting;
struct KeyboardLayoutSetting;
// I dont like this but i am too stupid and tired to think
struct MouseSensitivitySetting;
struct MouseForceNoAccelSetting;

impl SettingLine for MonitorSetting {
    fn prefix(&self) -> &str {
        MONITOR_SETTING_PREFIX
    }

    fn extract_key(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if let Some(setting) = trimmed.strip_prefix(self.prefix())
            && let Some(comma_pos) = setting.find(',')
        {
            return Some(setting[..comma_pos].to_string());
        }
        None
    }
}

impl SettingLine for KeyboardLayoutSetting {
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

impl SettingLine for MouseSensitivitySetting {
    fn prefix(&self) -> &str {
        MOUSE_SENSITIVITY_PREFIX
    }

    fn extract_key(&self, line: &str) -> Option<String> {
        if line.trim().starts_with(self.prefix()) {
            // For mouse sensitivity, we use a constant key since there's only one
            Some("sensitivity".to_string())
        } else {
            None
        }
    }
}

impl SettingLine for MouseForceNoAccelSetting {
    fn prefix(&self) -> &str {
        MOUSE_FORCE_NO_ACCEL_PREFIX
    }

    fn extract_key(&self, line: &str) -> Option<String> {
        if line.trim().starts_with(self.prefix()) {
            // For mouse force_no_accel, we use a constant key since there's only one
            Some("force_no_accel".to_string())
        } else {
            None
        }
    }
}

/// Registry of all known setting line types
pub fn get_setting_handlers() -> Vec<Box<dyn SettingLine>> {
    vec![
        Box::new(MonitorSetting),
        Box::new(KeyboardLayoutSetting),
        Box::new(MouseSensitivitySetting),
        Box::new(MouseForceNoAccelSetting),
    ]
}

/// Create the overrides setting file for hyprland.
/// This file is created at `~/.config/hypr/conf-overrides.conf`
/// If the file already exists, it will not be overwritten.
/// This function will only run once if the file does not exist.
/// It will also edit the main file `~/.config/hypr/hyprland.conf` to include the overrides file as
/// a source file at the bottom of the main setting file to ensure that all exisiting
/// settings are overwritten but not removed.
pub fn create_overrides() -> anyhow::Result<()> {
    let home_dir = home_dir().ok_or_else(|| {
        anyhow::anyhow!("Could not determine home directory for the current user")
    })?;

    let hypr_setting_path = home_dir.join(HYPR_SETTING_PATH);
    let hypr_overrides_path = home_dir.join(HYPR_OVERRIDES_PATH);

    if !Path::new(&hypr_setting_path).exists() {
        return Err(anyhow::anyhow!(
            "Hyprland setting file not found at {}, Hyprland is either not installed or not configured",
            HYPR_SETTING_PATH
        ));
    }

    if !Path::new(&hypr_overrides_path).exists() {
        std::fs::write(&hypr_overrides_path, "# Hyprland setting overrides\n")?;

        // append the file source line to main conf
        let mut hypr_setting_file = std::fs::OpenOptions::new()
            .append(true)
            .open(&hypr_setting_path)?;

        writeln!(
            hypr_setting_file,
            "\n# Include overrides setting\nsource = ~/{}",
            HYPR_OVERRIDES_PATH
        )?;
    }

    Ok(())
}
