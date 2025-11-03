use std::path::Path;

use crate::monitor::MonitorMode;
use dirs::home_dir;
use std::io::Write;

const HYPR_CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
const HYPR_OVERRIDES_PATH: &str = ".config/hypr/conf-overrides.conf";

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
pub fn write_override_line(line: &str) -> anyhow::Result<()> {
    let home_dir = home_dir().ok_or_else(|| {
        anyhow::anyhow!("Could not determine home directory for the current user")
    })?;

    let hypr_overrides_path = home_dir.join(HYPR_OVERRIDES_PATH);

    let mut hypr_overrides_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&hypr_overrides_path)?;

    writeln!(hypr_overrides_file, "{}", line)?;

    Ok(())
}

// unsure if i want this
// matching on a key is nice but probably overkill for just monitor settings lmfao
// pub enum OverrideKey {
//     Monitor,
//     Keybind,
// }
//
// pub struct OverrideSetting {
//     pub key: OverrideKey,
//     pub value: String,
// }

/// Generate a monitor override string for hyprland configuration.
/// Currently just generates the basic one as i am a europoor and only have 1 monitor to test with.
pub fn monitor_override(monitor_name: String, settings: MonitorMode) -> String {
    // for now we want this, as I only have 1 monitor to test with so position settings are TODO
    let auto_position_string = format!("{}@{},auto,1", settings.resolution, settings.refresh_rate);

    format!("monitor={},{}", monitor_name, auto_position_string)
}
