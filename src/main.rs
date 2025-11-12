use std::io::Write;
use std::rc::Rc;

use dirs::home_dir;
use gpui::*;
use gpui_component::*;
use serde::Deserialize;
use std::path::Path;

mod setting;
mod setting_writer;
mod ui;
mod util;

use crate::setting_writer::{HYPR_OVERRIDES_PATH, HYPR_SETTING_PATH};
use crate::ui::keyboard_settings::KeyboardSettings;
use crate::ui::monitor_visualizer::MonitorVisualizer;
use crate::ui::mouse_settings::MouseSettings;
use crate::ui::section_container::{section_divider, section_title};
use crate::ui::sidebar::create_sidebar;
use crate::util::monitor;

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveSection {
    Monitors,
    Keyboard,
    Mouse,
}

impl ToString for ActiveSection {
    fn to_string(&self) -> String {
        match self {
            ActiveSection::Monitors => "Monitors".to_string(),
            ActiveSection::Keyboard => "Keyboard".to_string(),
            ActiveSection::Mouse => "Mouse".to_string(),
        }
    }
}

pub struct Hyprsetting {
    monitor_visualizer: Entity<MonitorVisualizer>,
    keyboard_settings: Entity<KeyboardSettings>,
    mouse_settings: Entity<MouseSettings>,
    active_section: ActiveSection,
}

impl Hyprsetting {
    pub fn set_active_section(&mut self, section: ActiveSection, cx: &mut Context<Self>) {
        self.active_section = section;
        cx.notify();
    }
}

impl Render for Hyprsetting {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_section = self.active_section;

        div()
            .size_full()
            .flex()
            .bg(cx.theme().background)
            .child(create_sidebar(active_section, cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .overflow_hidden()
                    .child(section_title(
                        format!("{} settings", active_section.to_string()),
                        cx,
                    ))
                    .child(section_divider(cx))
                    .child(match active_section {
                        ActiveSection::Monitors => div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(section_title("Monitors", cx))
                            .child(self.monitor_visualizer.clone()),
                        ActiveSection::Keyboard => div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(section_title("Keyboard & language", cx))
                            .child(self.keyboard_settings.clone()),
                        ActiveSection::Mouse => div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(section_title("Mouse", cx))
                            .child(self.mouse_settings.clone()),
                    }),
            )
    }
}

#[derive(Deserialize)]
struct ThemeFile {
    themes: Vec<ThemeConfig>,
}

pub fn init(cx: &mut App) {
    let theme_content = include_str!("../themes/rose-pine.json");
    let theme_file: ThemeFile = serde_json::from_str(theme_content).unwrap();

    if let Some(theme) = theme_file
        .themes
        .into_iter()
        .find(|t| t.name == "Rose Pine")
    {
        Theme::global_mut(cx).apply_config(&Rc::new(theme));
    }
}
fn main() {
    // first check if overrides file exists, if not create it.

    create_overrides().expect("Failed to create Hyprland overrides setting file");

    let app = Application::new();

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        init(cx);

        cx.spawn(async move |cx| {
            let window_options = WindowOptions {
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            };

            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| {
                    // Load monitors
                    let monitors = monitor::get_monitors().unwrap_or_default();

                    let monitor_visualizer =
                        cx.new(|cx| MonitorVisualizer::new(monitors.clone(), window, cx));

                    // let input_settings = cx.new(|cx| InputSettings::new(window, cx));
                    let keyboard_settings = cx.new(|cx| KeyboardSettings::new(window, cx));

                    let mouse_settings = cx.new(|cx| MouseSettings::new(window, cx));

                    Hyprsetting {
                        monitor_visualizer,
                        keyboard_settings,
                        mouse_settings,
                        active_section: ActiveSection::Monitors,
                    }
                });
                // Root component
                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}

/// Create the overrides setting file for hyprland.
/// This file is created at `~/.config/hypr/conf-overrides.conf`
/// If the file already exists, it will not be overwritten.
/// This function will only run once if the file does not exist.
/// It will also edit the main file `~/.config/hypr/hyprland.conf` to include the overrides file as
/// a source file at the bottom of the main setting file to ensure that all exisiting
/// settings are overwritten but not removed.
fn create_overrides() -> anyhow::Result<()> {
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
