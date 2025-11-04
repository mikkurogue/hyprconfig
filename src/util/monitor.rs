use std::process::Command;

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub current_resolution: String,
    pub current_refresh_rate: f32,
    pub position: (i32, i32),
    pub available_modes: Vec<MonitorMode>,
}

#[derive(Debug, Clone)]
pub struct MonitorMode {
    pub resolution: String,
    pub refresh_rate: f32,
}

pub fn get_monitors() -> anyhow::Result<Vec<MonitorInfo>> {
    let output = Command::new("hyprctl").args(["monitors", "all"]).output()?;

    let stdout = String::from_utf8(output.stdout)?;
    parse_monitors(&stdout)
}

fn parse_monitors(output: &str) -> anyhow::Result<Vec<MonitorInfo>> {
    let mut monitors = Vec::new();
    let mut current_monitor: Option<MonitorInfo> = None;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("Monitor ") {
            // Save previous monitor if exists
            if let Some(monitor) = current_monitor.take() {
                monitors.push(monitor);
            }

            // Parse monitor header: "Monitor DP-3 (ID 0):"
            if let Some(name_part) = line.strip_prefix("Monitor ")
                && let Some(id_start) = name_part.find("(ID ")
            {
                let name = name_part[..id_start]
                    .trim()
                    .trim_end_matches(' ')
                    .to_string();
                if let Some(id_end) = name_part.find("):") {
                    let id_str = &name_part[id_start + 4..id_end];
                    if let Ok(id) = id_str.parse::<u32>() {
                        current_monitor = Some(MonitorInfo {
                            id,
                            name,
                            current_resolution: String::new(),
                            current_refresh_rate: 0.0,
                            position: (0, 0),
                            available_modes: Vec::new(),
                        });
                    }
                }
            }
        } else if let Some(monitor) = current_monitor.as_mut() {
            // Parse resolution and refresh rate: "2560x1440@155.00000 at 0x0"
            if line.contains("@") && line.contains(" at ") {
                let parts: Vec<&str> = line.split(" at ").collect();
                if parts.len() == 2 {
                    // Parse resolution and refresh rate
                    let mode_parts: Vec<&str> = parts[0].split('@').collect();
                    if mode_parts.len() == 2 {
                        monitor.current_resolution = mode_parts[0].to_string();
                        if let Ok(rate) = mode_parts[1].parse::<f32>() {
                            monitor.current_refresh_rate = rate;
                        }
                    }

                    // Parse position
                    let pos_parts: Vec<&str> = parts[1].split('x').collect();
                    if pos_parts.len() == 2
                        && let (Ok(x), Ok(y)) =
                            (pos_parts[0].parse::<i32>(), pos_parts[1].parse::<i32>())
                    {
                        monitor.position = (x, y);
                    }
                }
            } else if line.starts_with("availableModes:") {
                // Parse available modes: "availableModes: 2560x1440@59.95Hz ..."
                let modes_str = line.strip_prefix("availableModes:").unwrap_or("").trim();
                for mode_str in modes_str.split_whitespace() {
                    if let Some((res, rate_str)) = mode_str.split_once('@')
                        && let Some(rate_num) = rate_str.strip_suffix("Hz")
                        && let Ok(rate) = rate_num.parse::<f32>()
                    {
                        monitor.available_modes.push(MonitorMode {
                            resolution: res.to_string(),
                            refresh_rate: rate,
                        });
                    }
                }
            }
        }
    }

    // Don't forget the last monitor
    if let Some(monitor) = current_monitor {
        monitors.push(monitor);
    }

    Ok(monitors)
}
