use std::{collections::HashSet, path::Path};

use crate::monitor::MonitorMode;
use dirs::home_dir;
use std::io::Write;

const HYPR_CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
const HYPR_OVERRIDES_PATH: &str = ".config/hypr/conf-overrides.conf";

const MONITOR_CONFIG_PREFIX: &str = "monitor=";
const KEYBOARD_LAYOUT_PREFIX: &str = "input:kb_layout=";
const MOUSE_SENSITIVITY_PREFIX: &str = "input:sensitivity=";
const MOUSE_FORCE_NO_ACCEL_PREFIX: &str = "input:force_no_accel=";

/// Configuration objects for specific things like devices in hyprland
#[derive(PartialEq)]
enum ConfigObjectKey {
    Device,
}

/// Configuration keys for device settings in hyprland, for now we support only the name and the
/// kb_layout.
enum DeviceConfigKey {
    Name,
    KbLayout,
}

/// Struct representing a device setting for hyprland configuration
/// specicially for keyboards
pub struct DeviceSetting {
    key: ConfigObjectKey,
    device_name: String,
    kb_layout: String,
}

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
// I dont like this but i am too stupid and tired to think
struct MouseSensitivityConfig;
struct MouseForceNoAccelConfig;

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

impl ConfigLine for MouseSensitivityConfig {
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

impl ConfigLine for MouseForceNoAccelConfig {
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

/// Registry of all known config line types
fn get_config_handlers() -> Vec<Box<dyn ConfigLine>> {
    vec![
        Box::new(MonitorConfig),
        Box::new(KeyboardLayoutConfig),
        Box::new(MouseSensitivityConfig),
        Box::new(MouseForceNoAccelConfig),
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
/// Format: monitor=name,resolution@refreshrate,position,scale
/// Example: monitor=DP-3,2560x1440@155,0x0,1
pub fn monitor_override(
    monitor_name: String,
    settings: MonitorMode,
    position: (i32, i32),
) -> String {
    let position_string = format!("{}x{}", position.0, position.1);
    let config_string = format!(
        "{}@{},{},1",
        settings.resolution, settings.refresh_rate, position_string
    );

    format!(
        "{}{},{}",
        MONITOR_CONFIG_PREFIX, monitor_name, config_string
    )
}

/// Generate a keyboard locale override string for hyprland configuration.
/// E.g., for locales {"us", "fi"}, generates "input:kb_layout=us,fi"
#[deprecated(
    since = "0.2.0",
    note = "Use set_keyboard_device_layout instead for per-device settings"
)]
pub fn locale_override(locale: HashSet<String>) -> String {
    let mut input_locale_setting = String::from("input:kb_layout=");

    let locales: Vec<&str> = locale.iter().map(|s| s.as_str()).collect();
    input_locale_setting.push_str(&locales.join(","));

    input_locale_setting
}

pub fn set_keyboard_device_layout(device: String, locale: String) -> String {
    let input_device = DeviceSetting {
        key: ConfigObjectKey::Device,
        device_name: device,
        kb_layout: locale,
    };

    if let Ok(config_str) = config_obj_str_builder(input_device) {
        return config_str;
    } else {
        // For now we can panic here but we may want to just bubble up later.
        panic!("Failed to build device configuration string");
    }
}

fn config_obj_str_builder(input_device: DeviceSetting) -> anyhow::Result<String> {
    if input_device.key == ConfigObjectKey::Device {
        let start = "device { ".to_string();
        let end = " }".to_string();

        let name = format!("    name = {}", input_device.device_name);
        let kb_layout = format!("    kb_layout = {}", input_device.kb_layout);

        return Ok(format!("{}\n{}\n{}\n{}", start, name, kb_layout, end));
    }

    Err(anyhow::anyhow!("Unsupported configuration object"))
}

/// Generate a mouse sensitivity override string for hyprland configuration.
pub fn mouse_sensitivity_override(sensitivity: f32) -> String {
    format!("{}{}", MOUSE_SENSITIVITY_PREFIX, sensitivity)
}

/// Generate a mouse force_no_accel override string for hyprland configuration.
pub fn mouse_force_no_accel_override(force_no_accel: bool) -> String {
    format!(
        "{}{}",
        MOUSE_FORCE_NO_ACCEL_PREFIX,
        if force_no_accel { 1 } else { 0 }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_extract_key() {
        let monitor_config = MonitorConfig;
        assert_eq!(
            monitor_config.extract_key("monitor=DP-3,2560x1440@155,0x0,1"),
            Some("DP-3".to_string())
        );
        assert_eq!(
            monitor_config.extract_key("  monitor=HDMI-A-1,1920x1080@60,1920x0,1  "),
            Some("HDMI-A-1".to_string())
        );
        assert_eq!(monitor_config.extract_key("# comment line"), None);
        assert_eq!(monitor_config.extract_key("some other config"), None);
    }

    #[test]
    fn test_monitor_override_with_position() {
        let result = monitor_override(
            "DP-3".to_string(),
            MonitorMode {
                resolution: "2560x1440".to_string(),
                refresh_rate: 155.0,
            },
            (0, 0),
        );
        assert_eq!(result, "monitor=DP-3,2560x1440@155,0x0,1");

        let result2 = monitor_override(
            "HDMI-A-1".to_string(),
            MonitorMode {
                resolution: "1920x1080".to_string(),
                refresh_rate: 60.0,
            },
            (1920, 0),
        );
        assert_eq!(result2, "monitor=HDMI-A-1,1920x1080@60,1920x0,1");

        // Test negative position (monitor above primary)
        let result3 = monitor_override(
            "DP-2".to_string(),
            MonitorMode {
                resolution: "1920x1080".to_string(),
                refresh_rate: 144.0,
            },
            (0, -1080),
        );
        assert_eq!(result3, "monitor=DP-2,1920x1080@144,0x-1080,1");
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
