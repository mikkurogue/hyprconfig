
use crate::setting_writer::SettingLine;

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
