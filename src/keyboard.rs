use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct HyprctlDevices {
    keyboards: Vec<Keyboard>,
}

#[derive(Debug, Deserialize)]
struct Keyboard {
    layout: String,
}

#[derive(Debug, Clone)]
pub struct LocaleInfo {
    pub code: String,
    pub label: String,
}

/// Parse the XKB base.lst file to get all supported keyboard layouts
pub fn get_available_locales() -> anyhow::Result<Vec<LocaleInfo>> {
    let content = std::fs::read_to_string("/usr/share/X11/xkb/rules/base.lst")?;

    let mut locales = Vec::new();
    let mut in_layout_section = false;

    for line in content.lines() {
        if line.trim() == "! layout" {
            in_layout_section = true;
            continue;
        }

        if in_layout_section && line.starts_with('!') {
            break;
        }

        if in_layout_section && line.starts_with("  ") {
            let trimmed = line.trim();
            if let Some((code, label)) = trimmed.split_once(char::is_whitespace) {
                locales.push(LocaleInfo {
                    code: code.trim().to_string(),
                    label: label.trim().to_string(),
                });
            }
        }
    }

    Ok(locales)
}

/// Get the currently configured keyboard locales from hyprctl
/// just parse the first keyboard's layout field as all keyboards are assumed to have the same
/// layouts
pub fn get_current_locales() -> anyhow::Result<HashSet<String>> {
    let output = Command::new("hyprctl").args(["devices", "-j"]).output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to execute hyprctl devices"));
    }

    let json_str = String::from_utf8(output.stdout)?;
    let devices: HyprctlDevices = serde_json::from_str(&json_str)?;

    let mut locales = HashSet::new();

    if devices.keyboards.is_empty() {
        return Ok(locales);
    }

    if let Some(kb) = devices.keyboards.first() {
        for locale in kb.layout.split(',') {
            let trimmed = locale.trim();
            if !trimmed.is_empty() {
                locales.insert(trimmed.to_string());
            }
        }
    };

    Ok(locales)
}
