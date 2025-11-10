use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::IndexPath;
use gpui_component::button::Button;
use gpui_component::dropdown::*;
use std::process::Command;

// use crate::setting::{monitor_override, write_override_line};
use crate::setting_writer::SettingWriter;
use crate::ui::tooltip::with_tooltip;
use crate::util::monitor::MonitorInfo;
use crate::util::monitor::MonitorMode;

const PADDING: f32 = 40.0;
const MIN_CANVAS_WIDTH: f32 = 600.0;
const MIN_CANVAS_HEIGHT: f32 = 400.0;
const OVERALL_SCALE: f32 = 0.25; // Scale down to 25% of calculated size

#[derive(Clone)]
struct MonitorBox {
    monitor: MonitorInfo,
    visual_x: f32,
    visual_y: f32,
    visual_width: f32,
    visual_height: f32,
}

pub struct MonitorVisualizer {
    monitors: Vec<MonitorBox>,
    scale_factor: f32,
    dragging_index: Option<usize>,
    last_mouse_pos: Point<Pixels>,
    canvas_width: f32,
    canvas_height: f32,
    offset_x: f32,
    offset_y: f32,
    selected_monitor_index: Option<usize>,
    mouse_down_pos: Point<Pixels>,
    did_drag: bool,
    resolution_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
    refresh_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
    available_resolutions: Vec<String>,
    available_refresh_rates: Vec<String>,
}

impl MonitorVisualizer {
    pub fn new(monitors: Vec<MonitorInfo>, _window: &mut Window, _cx: &mut Context<Self>) -> Self {
        if monitors.is_empty() {
            return Self {
                monitors: vec![],
                scale_factor: 1.0,
                dragging_index: None,
                last_mouse_pos: Point::default(),
                canvas_width: MIN_CANVAS_WIDTH,
                canvas_height: MIN_CANVAS_HEIGHT,
                offset_x: 0.0,
                offset_y: 0.0,
                selected_monitor_index: None,
                mouse_down_pos: Point::default(),
                did_drag: false,
                resolution_dropdown: None,
                refresh_dropdown: None,
                available_resolutions: vec![],
                available_refresh_rates: vec![],
            };
        }

        // Calculate bounding box based on current monitor positions
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for monitor in &monitors {
            let (width, height) = Self::parse_resolution(&monitor.current_resolution);
            min_x = min_x.min(monitor.position.0);
            max_x = max_x.max(monitor.position.0 + width);
            min_y = min_y.min(monitor.position.1);
            max_y = max_y.max(monitor.position.1 + height);
        }

        let total_width = (max_x - min_x) as f32;
        let total_height = (max_y - min_y) as f32;

        // Calculate scale to fit monitors nicely with padding
        let target_width = (total_width + 2.0 * PADDING) * OVERALL_SCALE;
        let target_height = (total_height + 2.0 * PADDING) * OVERALL_SCALE;

        // Use at least minimum canvas size (also scaled)
        let canvas_width = target_width.max(MIN_CANVAS_WIDTH * OVERALL_SCALE);
        let canvas_height = target_height.max(MIN_CANVAS_HEIGHT * OVERALL_SCALE);

        // Scale factor to convert real coordinates to visual coordinates
        let scale_factor = ((canvas_width - 2.0 * PADDING * OVERALL_SCALE) / total_width)
            .min((canvas_height - 2.0 * PADDING * OVERALL_SCALE) / total_height)
            .min(0.3 * OVERALL_SCALE);

        // Calculate the scaled dimensions of the monitor layout
        let scaled_layout_width = total_width * scale_factor;
        let scaled_layout_height = total_height * scale_factor;

        // Center the layout in the canvas
        let offset_x = (canvas_width - scaled_layout_width) / 2.0 - (min_x as f32 * scale_factor);
        let offset_y = (canvas_height - scaled_layout_height) / 2.0 - (min_y as f32 * scale_factor);

        let monitor_boxes = monitors
            .into_iter()
            .map(|m| {
                let (width, height) = Self::parse_resolution(&m.current_resolution);
                // Position based on actual monitor position from hyprctl
                let visual_x = (m.position.0 as f32 * scale_factor) + offset_x;
                let visual_y = (m.position.1 as f32 * scale_factor) + offset_y;
                let visual_width = width as f32 * scale_factor;
                let visual_height = height as f32 * scale_factor;

                MonitorBox {
                    monitor: m,
                    visual_x,
                    visual_y,
                    visual_width,
                    visual_height,
                }
            })
            .collect();

        Self {
            monitors: monitor_boxes,
            scale_factor,
            dragging_index: None,
            last_mouse_pos: Point::default(),
            canvas_width,
            canvas_height,
            offset_x,
            offset_y,
            selected_monitor_index: None,
            mouse_down_pos: Point::default(),
            did_drag: false,
            resolution_dropdown: None,
            refresh_dropdown: None,
            available_resolutions: vec![],
            available_refresh_rates: vec![],
        }
    }

    fn update_dropdowns_for_monitor(
        &mut self,
        idx: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(monitor_box) = self.monitors.get(idx) {
            let monitor = &monitor_box.monitor;

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

            self.available_resolutions = resolutions;
            self.available_refresh_rates = refresh_rates;
            self.resolution_dropdown = Some(resolution_dropdown);
            self.refresh_dropdown = Some(refresh_dropdown);
        }
    }

    fn parse_resolution(resolution: &str) -> (i32, i32) {
        if let Some((w, h)) = resolution.split_once('x') {
            (w.parse().unwrap_or(1920), h.parse().unwrap_or(1080))
        } else {
            (1920, 1080)
        }
    }

    fn calculate_actual_position(&self, visual_x: f32, visual_y: f32) -> (i32, i32) {
        // Convert visual position back to actual Hyprland coordinates
        let actual_x = ((visual_x - self.offset_x) / self.scale_factor).round() as i32;
        let actual_y = ((visual_y - self.offset_y) / self.scale_factor).round() as i32;
        (actual_x, actual_y)
    }

    fn print_monitor_positions(&self) {
        println!("\n=== Monitor Positions ===");
        for monitor_box in &self.monitors {
            let position =
                self.calculate_actual_position(monitor_box.visual_x, monitor_box.visual_y);
            let is_primary = position == (0, 0);
            println!(
                "{} (ID: {}): {}x{} {}",
                monitor_box.monitor.name,
                monitor_box.monitor.id,
                position.0,
                position.1,
                if is_primary { "[PRIMARY]" } else { "" }
            );
        }
        println!("========================\n");
    }

    fn render_monitor_details_panel(
        &self,
        monitor: &MonitorInfo,
        theme: &gpui_component::theme::Theme,
    ) -> impl IntoElement {
        let monitor_position = monitor.position;
        let monitor_name = monitor.name.clone();
        let resolutions = self.available_resolutions.clone();
        let refresh_rates = self.available_refresh_rates.clone();
        let resolution_dropdown = self.resolution_dropdown.clone();
        let refresh_dropdown = self.refresh_dropdown.clone();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_color(theme.foreground.opacity(0.7))
                            .text_size(px(12.0))
                            .child("Resolution:"),
                    )
                    .when_some(self.resolution_dropdown.clone(), |this, dropdown| {
                        this.child(Dropdown::new(&dropdown).min_w(px(200.0)))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_color(theme.foreground.opacity(0.7))
                            .text_size(px(12.0))
                            .child("Refresh Rate:"),
                    )
                    .when_some(self.refresh_dropdown.clone(), |this, dropdown| {
                        this.child(Dropdown::new(&dropdown).min_w(px(200.0)))
                    }),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .mt_2()
                    .child(
                        div()
                            .text_color(theme.foreground.opacity(0.7))
                            .child("Position:"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground)
                            .child(format!("{}x{}", monitor_position.0, monitor_position.1)),
                    ),
            )
            .child(
                div().flex().justify_center().mt_3().child(
                    Button::new("apply-monitor-setting")
                        .label("Apply Setting")
                        .on_click(move |_, _, cx| {
                            Self::apply_monitor_setting(
                                &monitor_name,
                                monitor_position,
                                &resolutions,
                                &refresh_rates,
                                resolution_dropdown.clone(),
                                refresh_dropdown.clone(),
                                cx,
                            );
                        }),
                ),
            )
            .when(monitor_position == (0, 0), |this| {
                this.child(
                    div().flex().justify_center().mt_2().child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x4a7c59))
                            .text_color(rgb(0xeceff4))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .child("PRIMARY MONITOR"),
                    ),
                )
            })
    }

    fn apply_monitor_setting(
        monitor_name: &str,
        position: (i32, i32),
        resolutions: &[String],
        refresh_rates: &[String],
        resolution_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
        refresh_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
        cx: &mut App,
    ) {
        let Some((res_dropdown, ref_dropdown)) = resolution_dropdown.zip(refresh_dropdown) else {
            return;
        };

        let selected_res_idx = res_dropdown.read(cx).selected_index(cx);
        let selected_refresh_idx = ref_dropdown.read(cx).selected_index(cx);

        let Some((res_idx, refresh_idx)) = selected_res_idx.zip(selected_refresh_idx) else {
            return;
        };

        let resolution = &resolutions[res_idx.row];
        let refresh_rate_str = &refresh_rates[refresh_idx.row];

        let refresh_rate: f32 = refresh_rate_str
            .trim_end_matches("Hz")
            .parse()
            .unwrap_or(60.0);

        println!(
            "Applying: {} @ {}Hz at {}x{} to {}",
            resolution, refresh_rate, position.0, position.1, monitor_name
        );

        let monitor_setting_value = format!(
            "{},{}@{},{}x{},1",
            monitor_name, resolution, refresh_rate, position.0, position.1
        );
        if let Err(e) = SettingWriter::build_single("monitor=", monitor_setting_value.clone())
            .and_then(|w| w.write())
        {
            println!("Failed to write override: {}", e);
        }

        let setting_value = format!(
            "{},{}@{},{}x{},1",
            monitor_name, resolution, refresh_rate, position.0, position.1
        );

        match Command::new("hyprctl")
            .args(["keyword", "monitor", &setting_value])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    println!("✓ Monitor setting applied successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("✗ Failed to apply monitor setting: {}", stderr);
                }
            }
            Err(e) => {
                println!("✗ Failed to execute hyprctl: {}", e);
            }
        }
    }

    fn apply_monitor_setting_immediately(&self, monitor_box: &MonitorBox) {
        let setting_value = format!(
            "{},{}@{},{}x{},1",
            monitor_box.monitor.name,
            monitor_box.monitor.current_resolution,
            monitor_box.monitor.current_refresh_rate,
            monitor_box.monitor.position.0,
            monitor_box.monitor.position.1
        );

        println!("Applying monitor position via hyprctl: {}", setting_value);

        match Command::new("hyprctl")
            .args(["keyword", "monitor", &setting_value])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    println!("✓ Monitor position applied successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("✗ Failed to apply monitor position: {}", stderr);
                }
            }
            Err(e) => {
                println!("✗ Failed to execute hyprctl: {}", e);
            }
        }
    }
}

impl Render for MonitorVisualizer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let theme_colors = theme.clone();
        let foreground = theme.foreground;
        let foreground_muted = theme.foreground.opacity(0.7);
        let background = theme.background;
        let border = theme.border;
        let selected_monitor = self
            .selected_monitor_index
            .and_then(|idx| self.monitors.get(idx))
            .map(|m| m.monitor.clone());

        div()
            .relative()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                with_tooltip(
                    "Monitor positions are calculated as pixel coordinates relative to the primary monitor at 0x0. Drag monitors to position them. For now we do not yet supporting snapping, so make sure there are no gaps!",
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_color(foreground)
                        .text_size(px(18.0))
                        .child("Monitor Layout Visualizer"),
                    cx,
                )
            )
            .child(
                div()
                    .text_color(foreground_muted)
                    .text_size(px(12.0))
                    .child("Drag secondary monitors to position them. Primary monitor (green) is fixed at 0x0."),
            )
            .child(
                div()
                  .relative()
                  .w(px(self.canvas_width))
                  .h(px(self.canvas_height))
                  .bg(background)
                  .border_1()
                  .border_color(border)
                  .rounded_md()
                  .overflow_hidden()
                  .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                      if let Some(idx) = this.dragging_index {
                          // Don't allow dragging the primary monitor
                          if let Some(monitor) = this.monitors.get(idx)
                              && monitor.monitor.position != (0, 0) {
                                  let delta_x: f32 = (event.position.x - this.last_mouse_pos.x).into();
                                  let delta_y: f32 = (event.position.y - this.last_mouse_pos.y).into();

                                  // Mark that we've moved
                                  if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                                      this.did_drag = true;
                                  }

                                  if let Some(monitor) = this.monitors.get_mut(idx) {
                                      monitor.visual_x += delta_x;
                                      monitor.visual_y += delta_y;
                                  }

                                  this.last_mouse_pos = event.position;
                                  cx.notify();
                              }
                      }
                  }))
                  .on_mouse_up(MouseButton::Left, cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                      if let Some(idx) = this.dragging_index {
                          // Apply snapping before releasing
                          snap(&mut this.monitors, idx);
                          this.dragging_index = None;
                          this.print_monitor_positions();
                          cx.notify();
                        }
                    }))
                    .children(self.monitors.iter().enumerate().map(|(idx, monitor_box)| {
                        let is_primary = monitor_box.monitor.position == (0, 0);
                        let monitor_name = monitor_box.monitor.name.clone();
                        let monitor_id = monitor_box.monitor.id;
                        let visual_x = monitor_box.visual_x;
                        let visual_y = monitor_box.visual_y;
                        let is_dragging = self.dragging_index == Some(idx);

                        div()
                            .absolute()
                            .left(px(visual_x))
                            .top(px(visual_y))
                            .w(px(monitor_box.visual_width))
                            .h(px(monitor_box.visual_height))
                            .bg(if is_primary {
                                rgb(0x4a7c59)
                            } else {
                                rgb(0x3b4252)
                            })
                            .border_2()
                            .border_color(if is_dragging {
                                rgb(0x88c0d0)
                            } else if is_primary {
                                rgb(0x5e8d6f)
                            } else {
                                rgb(0x4c566a)
                            })
                            .rounded_md()
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                                // Store initial position
                                this.mouse_down_pos = event.position;
                                this.did_drag = false;

                                // For secondary monitors, start dragging
                                if !is_primary {
                                    this.dragging_index = Some(idx);
                                    this.last_mouse_pos = event.position;
                                }
                                cx.notify();
                            }))
                            .on_mouse_up(MouseButton::Left, cx.listener(move |this, _event: &MouseUpEvent, window, cx| {
                                // If we dragged, don't open popup
                                if !this.did_drag {
                                    // It was a click - toggle popup
                                    if this.selected_monitor_index == Some(idx) {
                                        this.selected_monitor_index = None;
                                    } else {
                                        this.selected_monitor_index = Some(idx);
                                        // Initialize dropdowns for this monitor
                                        this.update_dropdowns_for_monitor(idx, window, cx);
                                    }
                                } else {
                                    // Update the monitor's position after dragging
                                    if let Some(monitor_box) = this.monitors.get(idx) {
                                        // Calculate new position first (immutable borrow)
                                        let new_position = this.calculate_actual_position(
                                            monitor_box.visual_x,
                                            monitor_box.visual_y,
                                        );

                                        // Now update with mutable borrow
                                        if let Some(monitor_box) = this.monitors.get_mut(idx) {
                                            monitor_box.monitor.position = new_position;

                                            // Write the new position to setting file
                                            let monitor_setting_value = format!(
                                                "{},{}@{},{}x{},1",
                                                monitor_box.monitor.name,
                                                monitor_box.monitor.current_resolution,
                                                monitor_box.monitor.current_refresh_rate,
                                                new_position.0,
                                                new_position.1
                                            );
                                            if let Err(e) = SettingWriter::build_single("monitor=", monitor_setting_value.clone())
                                                .and_then(|w| w.write()) {
                                                println!("Failed to write override: {}", e);
                                            }

                                            // Apply immediately via hyprctl
                                            let monitor_box_clone = monitor_box.clone();
                                            this.apply_monitor_setting_immediately(&monitor_box_clone);
                                        }
                                    }
                                    // Print positions after dragging
                                    this.print_monitor_positions();
                                }

                                // Reset drag state
                                this.dragging_index = None;
                                this.did_drag = false;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .size_full()
                                    .text_color(rgb(0xeceff4))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::BOLD)
                                            .child(monitor_name.clone()),
                                    )
                                    .child(div().text_size(px(10.0)).child(format!("ID: {}", monitor_id)))
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .child(monitor_box.monitor.current_resolution.clone()),
                                    )
                                    .when(is_primary, |this| {
                                        this.child(
                                            div()
                                                .text_size(px(9.0))
                                                .text_color(rgb(0xa3be8c))
                                                .child("PRIMARY"),
                                        )
                                    }),
                            )
                    })),
            )
            .child(
                div()
                    .text_color(foreground_muted)
                    .text_size(px(11.0))
                    .child(format!("Scale factor: {:.4}", self.scale_factor)),
            )
            .when_some(selected_monitor, |this, monitor| {
                this.child(
                    // Backdrop overlay
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x00000088))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                            this.selected_monitor_index = None;
                            cx.notify();
                        }))
                        .child(
                            // Popup content
                            div()
                                .bg(theme_colors.background)
                                .border_1()
                                .border_color(theme_colors.border)
                                .rounded_lg()
                                .p_6()
                                .min_w(px(300.0))
                                .shadow_lg()
                                .on_mouse_down(MouseButton::Left, |_event: &MouseDownEvent, _window, cx| {
                                    // Stop propagation to prevent closing when clicking inside
                                    cx.stop_propagation();
                                })
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_3()
                                        .child(
                                            div()
                                                .flex()
                                                .justify_between()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_size(px(16.0))
                                                        .text_color(theme_colors.foreground)
                                                        .child(format!("{} (ID: {})", monitor.name, monitor.id)),
                                                )
                                                .child(
                                                    div()
                                                        .cursor_pointer()
                                                        .text_size(px(20.0))
                                                        .text_color(theme_colors.foreground.opacity(0.7))
                                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                                                            this.selected_monitor_index = None;
                                                            cx.notify();
                                                        }))
                                                        .child("×"),
                                                ),
                                        )
                        .child(
                            div()
                                .h_px()
                                .bg(theme_colors.border),
                        )
                        .child(self.render_monitor_details_panel(&monitor, &theme_colors)),
                ),
        ),
    )
})
    }
}

const SNAP_THRESHOLD: f32 = 20.0;

fn find_closest(dragged_monitor: &MonitorBox, other_monitors: &[MonitorBox]) -> Option<(f32, f32)> {
    let mut min_distance = f32::MAX;
    let mut snap_offset_x = 0.0;
    let mut snap_offset_y = 0.0;

    let dragged_right = dragged_monitor.visual_x + dragged_monitor.visual_width;
    let dragged_bottom = dragged_monitor.visual_y + dragged_monitor.visual_height;

    for other in other_monitors {
        let other_right = other.visual_x + other.visual_width;
        let other_bottom = other.visual_y + other.visual_height;

        // Check all edge combinations
        let snap_points = [
            // Snap left edge to other's right edge
            (
                other_right - dragged_monitor.visual_x,
                0.0,
                other_right,
                dragged_monitor.visual_y,
            ),
            // Snap right edge to other's left edge
            (
                other.visual_x - dragged_right,
                0.0,
                other.visual_x - dragged_monitor.visual_width,
                dragged_monitor.visual_y,
            ),
            // Snap top edge to other's bottom edge
            (
                0.0,
                other_bottom - dragged_monitor.visual_y,
                dragged_monitor.visual_x,
                other_bottom,
            ),
            // Snap bottom edge to other's top edge
            (
                0.0,
                other.visual_y - dragged_bottom,
                dragged_monitor.visual_x,
                other.visual_y - dragged_monitor.visual_height,
            ),
        ];

        for (dx, dy, new_x, new_y) in snap_points {
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < min_distance && distance < SNAP_THRESHOLD {
                min_distance = distance;
                snap_offset_x = new_x - dragged_monitor.visual_x;
                snap_offset_y = new_y - dragged_monitor.visual_y;
            }
        }

        // Also check for alignment (same x or y while snapping the other axis)
        // Align top edges
        if (dragged_monitor.visual_y - other.visual_y).abs() < SNAP_THRESHOLD {
            snap_offset_y = other.visual_y - dragged_monitor.visual_y;
        }
        // Align bottom edges
        if (dragged_bottom - other_bottom).abs() < SNAP_THRESHOLD {
            snap_offset_y = other_bottom - dragged_bottom;
        }
        // Align left edges
        if (dragged_monitor.visual_x - other.visual_x).abs() < SNAP_THRESHOLD {
            snap_offset_x = other.visual_x - dragged_monitor.visual_x;
        }
        // Align right edges
        if (dragged_right - other_right).abs() < SNAP_THRESHOLD {
            snap_offset_x = other_right - dragged_right;
        }
    }

    if min_distance < SNAP_THRESHOLD || snap_offset_x.abs() > 0.1 || snap_offset_y.abs() > 0.1 {
        Some((snap_offset_x, snap_offset_y))
    } else {
        None
    }
}

fn snap(monitors: &mut [MonitorBox], dragging_index: usize) {
    if dragging_index >= monitors.len() {
        return;
    }

    let dragged = monitors[dragging_index].clone();
    let others: Vec<MonitorBox> = monitors
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != dragging_index)
        .map(|(_, m)| m.clone())
        .collect();

    if let Some((offset_x, offset_y)) = find_closest(&dragged, &others) {
        monitors[dragging_index].visual_x += offset_x;
        monitors[dragging_index].visual_y += offset_y;
    }
}
