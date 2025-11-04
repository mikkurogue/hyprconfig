use gpui::*;
use gpui_component::button::Button;
use gpui_component::dropdown::*;

use gpui_component::{IndexPath, StyledExt};

use crate::conf::{monitor_override, write_override_line};
use crate::util::monitor::{MonitorInfo, MonitorMode};

pub struct MonitorSettings {
    monitor_id: u32,
    monitor_name: SharedString,
    resolutions: Vec<String>,
    refresh_rates: Vec<String>,
    resolution_dropdown: Entity<DropdownState<Vec<String>>>,
    refresh_dropdown: Entity<DropdownState<Vec<String>>>,
    // position_dropdown: Entity<DropdownState<Vec<String>>>,
}

impl MonitorSettings {
    pub fn new(monitor: MonitorInfo, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Get unique resolutions
        let mut resolutions: Vec<String> = monitor
            .available_modes
            .iter()
            .map(|m| m.resolution.clone())
            .collect();
        resolutions.sort();
        resolutions.dedup();

        let current_res_idx = resolutions
            .iter()
            .position(|r| r == &monitor.current_resolution);

        let resolution_dropdown = cx.new(|cx| {
            DropdownState::new(
                resolutions.clone(),
                current_res_idx.map(IndexPath::new),
                window,
                cx,
            )
        });

        // Get refresh rates for current resolution
        let refresh_rates: Vec<String> = monitor
            .available_modes
            .iter()
            .filter(|m| m.resolution == monitor.current_resolution)
            .map(|m| format!("{:.2}Hz", m.refresh_rate))
            .collect();

        let current_refresh_str = format!("{:.2}Hz", monitor.current_refresh_rate);
        let current_refresh_idx = refresh_rates.iter().position(|r| r == &current_refresh_str);

        let refresh_dropdown = cx.new(|cx| {
            DropdownState::new(
                refresh_rates.clone(),
                current_refresh_idx.map(IndexPath::new),
                window,
                cx,
            )
        });

        // Position options (placeholder)
        // let positions = vec![
        //     "Left".to_string(),
        //     "Right".to_string(),
        //     "Top".to_string(),
        //     "Bottom".to_string(),
        //     format!("{}x{}", monitor.position.0, monitor.position.1),
        // ];
        //
        // let position_dropdown = cx.new(|cx| {
        //     DropdownState::new(
        //         positions,
        //         Some(IndexPath::new(4)), // Default to current position
        //         window,
        //         cx,
        //     )
        // });

        Self {
            monitor_id: monitor.id,
            monitor_name: monitor.name.into(),
            resolutions,
            refresh_rates,
            resolution_dropdown,
            refresh_dropdown,
            // position_dropdown,
        }
    }
}

impl Render for MonitorSettings {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let monitor_id = self.monitor_id;
        let monitor_name = self.monitor_name.clone();
        let resolutions = self.resolutions.clone();
        let refresh_rates = self.refresh_rates.clone();
        let resolution_dropdown = self.resolution_dropdown.clone();
        let refresh_dropdown = self.refresh_dropdown.clone();

        div()
            .v_flex()
            .gap_2()
            .p_4()
            .border_1()
            .border_color(rgb(0x404040))
            .rounded_lg()
            .child(
                div()
                    .font_weight(FontWeight::BOLD)
                    .child(format!("{} (ID: {})", self.monitor_name, self.monitor_id)),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)).child("Resolution:"))
                    .child(Dropdown::new(&self.resolution_dropdown).min_w(px(200.0))),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)).child("Refresh Rate:"))
                    .child(Dropdown::new(&self.refresh_dropdown).min_w(px(200.0))),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)))
                    .child(
                        Button::new("apply-monitor-settings")
                            .label("Apply monitor config")
                            .on_click(move |_, _, cx| {
                                // Read current selected indices from the dropdown entities
                                let selected_res_idx =
                                    resolution_dropdown.read(cx).selected_index(cx);
                                let selected_refresh_idx =
                                    refresh_dropdown.read(cx).selected_index(cx);

                                // Get the actual string values if they exist
                                if let (Some(res_idx), Some(refresh_idx)) =
                                    (selected_res_idx, selected_refresh_idx)
                                {
                                    let resolution = &resolutions[res_idx.row];
                                    let refresh_rate_str = &refresh_rates[refresh_idx.row];

                                    let refresh_rate: f32 = refresh_rate_str
                                        .trim_end_matches("Hz")
                                        .parse()
                                        .unwrap_or(60.0);

                                    // DEBUG
                                    println!(
                                        "Apply monitor settings clicked for monitor: {} (ID: {})",
                                        monitor_name, monitor_id
                                    );
                                    // DEBUG
                                    println!("Selected resolution: {}", resolution);
                                    // DEBUG
                                    println!("Selected refresh rate: {}", refresh_rate);

                                    let override_str = monitor_override(
                                        monitor_name.to_string(),
                                        MonitorMode {
                                            resolution: resolution.clone(),
                                            refresh_rate,
                                        },
                                    );

                                    // DEBUG
                                    println!("Generated override string: {}", override_str);

                                    write_override_line(&override_str).unwrap_or_else(|e| {
                                        println!("Failed to write override line: {}", e);
                                    });
                                } else {
                                    println!("No selection made in dropdowns");
                                }
                            }),
                    ),
            )
        // disable position for now - see reasoning in @src/conf.rs monitor_override
        // .child(
        //     div()
        //         .h_flex()
        //         .gap_4()
        //         .items_center()
        //         .child(div().min_w(px(120.0)).child("Position:"))
        //         .child(Dropdown::new(&self.position_dropdown).min_w(px(200.0))),
        // )
    }
}
