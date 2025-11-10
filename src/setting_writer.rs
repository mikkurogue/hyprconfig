use std::fs;

use dirs::home_dir;

use crate::setting::get_setting_handlers;

pub const HYPR_SETTING_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPR_OVERRIDES_PATH: &str = ".config/hypr/conf-overrides.conf";

pub struct SettingWriter {
    setting_line: (SettingObjectKey, String),
}

impl SettingWriter {
    /// Setting builder for device setting. Currently only supports keyboard layout settings.
    pub fn build(input_device: DeviceSetting) -> anyhow::Result<Self> {
        if input_device.key == SettingObjectKey::Device {
            let start = "device { ".to_string();
            let end = " }".to_string();

            let name = format!("    name = {}", input_device.device_name);
            let kb_layout = format!("    kb_layout = {}", input_device.kb_layout);

            return Ok(SettingWriter {
                setting_line: (
                    SettingObjectKey::Device,
                    format!("{}\n{}\n{}\n{}", start, name, kb_layout, end),
                ),
            });
        }

        Err(anyhow::anyhow!(
            "Unsupported setting object, for non-device specific settings use `setting::write_override_line` function instead - this writer will be updated to handle single line settings in the future"
        ))
    }

    pub fn build_single<T: std::fmt::Display>(prefix: &str, value: T) -> anyhow::Result<Self> {
        Ok(SettingWriter {
            setting_line: (SettingObjectKey::SingleLine, format!("{}{}", prefix, value)),
        })
    }

    /// Write the setting override file if it doesn't exist, and append the source line to the
    /// main hyprland setting file.
    pub fn write(self) -> anyhow::Result<()> {
        let home_dir = home_dir().ok_or_else(|| {
            anyhow::anyhow!("Could not determine home directory for the current user")
        })?;

        let hypr_overrides_path = home_dir.join(HYPR_OVERRIDES_PATH);

        // Read existing content
        let content = std::fs::read_to_string(&hypr_overrides_path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        let handlers = get_setting_handlers();
        let mut replaced = false;

        // Try to find a handler that matches this line
        for handler in &handlers {
            if let Some(new_key) = handler.extract_key(self.setting_line.1.as_str())
                && handler.should_replace()
            {
                // Find and replace existing line with the same key
                for existing_line in lines.iter_mut() {
                    if let Some(existing_key) = handler.extract_key(existing_line)
                        && existing_key == new_key
                    {
                        *existing_line = self.setting_line.1.to_string();
                        replaced = true;
                        break;
                    }
                }
                break;
            }
        }

        // If not replaced, append the new line
        if !replaced {
            lines.push(self.setting_line.1.to_string());
        }

        // Write back to file
        let updated_content = lines.join("\n") + "\n";
        fs::write(&hypr_overrides_path, updated_content)?;

        Ok(())
    }
}

/// Setting objects for specific things like devices in hyprland
#[derive(PartialEq)]
pub enum SettingObjectKey {
    Device,
    SingleLine,
}

/// Struct representing a device setting for hyprland setting
/// specicially for keyboards
pub struct DeviceSetting {
    pub key: SettingObjectKey,
    pub device_name: String,
    pub kb_layout: String,
}

/// Trait for setting lines that can be overridden in the setting file
pub trait SettingLine {
    /// Get the prefix that identifies this type of setting line
    fn prefix(&self) -> &str;

    /// Extract the identifier/key from a setting line (e.g., monitor name, setting key)
    /// Returns None if the line doesn't match this setting type
    fn extract_key(&self, line: &str) -> Option<String>;

    /// Check if this line should replace an existing line with the same key
    fn should_replace(&self) -> bool {
        true
    }
}
