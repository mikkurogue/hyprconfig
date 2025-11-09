use std::rc::Rc;

use gpui::*;
use gpui_component::*;
use serde::Deserialize;

mod setting;
mod setting_writer;
mod ui;
mod util;

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
    let theme_content = include_str!("../themes/catppuccin.json");
    let theme_file: ThemeFile = serde_json::from_str(theme_content).unwrap();

    if let Some(theme) = theme_file
        .themes
        .into_iter()
        .find(|t| t.name == "Catppuccin Mocha")
    {
        Theme::global_mut(cx).apply_config(&Rc::new(theme));
    }
}
fn main() {
    // first check if overrides file exists, if not create it.

    setting::create_overrides().expect("Failed to create Hyprland overrides setting file");

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
