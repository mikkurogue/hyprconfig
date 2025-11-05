use gpui::App;
use gpui::*;
use gpui_component::*;

mod conf;
mod ui;
mod util;

use crate::ui::section_container::{
    main_container, section_divider, section_sub_container, section_title,
};
use crate::ui::{input_settings::InputSettings, monitor_settings::MonitorSettings};
use crate::util::monitor;

pub struct Hyprconfig {
    monitor_settings: Vec<Entity<MonitorSettings>>,
    input_settings: Entity<InputSettings>,
}

impl Render for Hyprconfig {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        main_container()
            .child(section_title("Hyprland configuration tool"))
            .child(section_divider())
            .child(section_title("Monitors"))
            .child(section_sub_container().children(self.monitor_settings.iter().cloned()))
            .child(section_divider())
            .child(section_title("Input"))
            .child(self.input_settings.clone())
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

                    let input_settings = cx.new(|cx| InputSettings::new(window, cx));

                    Hyprconfig {
                        monitor_settings,
                        input_settings,
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
