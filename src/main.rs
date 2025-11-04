use gpui::App;
use gpui::*;
use gpui_component::*;

mod conf;
mod ui;
mod util;

use crate::ui::{keyboard_settings::KeyboardSettings, monitor_settings::MonitorSettings};
use crate::util::monitor;

pub struct Hyprconfig {
    monitor_settings: Vec<Entity<MonitorSettings>>,
    keyboard_settings: Entity<KeyboardSettings>,
}

impl Render for Hyprconfig {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_4()
            .size_full()
            .bg(transparent_white())
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Hyprland configuration helper"),
            )
            .child(div().w_full().h_1().rounded_sm().bg(rgb(0xf6f6f6)))
            .child(div().font_normal().child("Monitors"))
            .children(self.monitor_settings.iter().cloned())
            .child(div().w_full().h_1().rounded_sm().bg(rgb(0xf6f6f6)))
            .child(div().font_normal().child("Input"))
            .child(self.keyboard_settings.clone())
    }
}

pub fn init(cx: &mut App) {
    // gpui-component has built-in themes
    let theme_name = "Default Dark";

    // Try to load the built-in theme
    if let Some(theme) = ThemeRegistry::global(cx).themes().get(theme_name).cloned() {
        Theme::global_mut(cx).apply_config(&theme);
    } else {
        eprintln!("Theme '{}' not found. Available themes:", theme_name);
        for name in ThemeRegistry::global(cx).themes().keys() {
            eprintln!("  - {}", name);
        }
    }
}

fn main() {
    // first check if overrides file exists, if not create it.

    conf::create_overrides().expect("Failed to create Hyprland overrides configuration file");

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
                    let monitor_settings: Vec<Entity<MonitorSettings>> = monitors
                        .into_iter()
                        .map(|monitor| cx.new(|cx| MonitorSettings::new(monitor, window, cx)))
                        .collect();

                    let keyboard_settings = cx.new(|cx| KeyboardSettings::new(window, cx));

                    Hyprconfig {
                        monitor_settings,
                        keyboard_settings,
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
