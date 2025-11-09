use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::StyledExt;
use gpui_component::button::Button;
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use gpui_component::switch::Switch;

use crate::setting::{mouse_force_no_accel_override, mouse_sensitivity_override, write_override_line};
use crate::ui::{section_container::section_container, tooltip::with_tooltip};
use crate::util::mouse::{get_accel_setting, get_current_sensitivity};

pub struct MouseSettings {
    force_no_accel_checked: bool,
    mouse_sensitivity_slider: Entity<SliderState>,
    current_sensitivity: f32,
}

// helper to convert sensitivity to slider value and back
// i think gpui_component has a issue with the negative floats?
fn sensitivity_to_slider(sens: f32) -> f32 {
    // Convert -1.0..1.0 to 0.0..100.0
    (sens + 1.0) * 50.0
}

fn slider_to_sensitivity(slider: f32) -> f32 {
    // Convert 0.0..100.0 to -1.0..1.0
    (slider / 50.0) - 1.0
}

impl MouseSettings {
    pub fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let current_sens = get_current_sensitivity().unwrap_or(0.0);
        let accel_setting = get_accel_setting().unwrap_or(false);

        println!(
            "DEBUG: Initializing slider with sensitivity: {}",
            current_sens
        );

        let mouse_sensitivity_slider = cx.new(|_cx| {
            SliderState::new().min(0.0).max(100.0).step(2.5) // 0.05 * 50
        });

        // Set value after creation
        mouse_sensitivity_slider.update(cx, |state, cx| {
            state.set_value(sensitivity_to_slider(current_sens), window, cx);
        });

        cx.subscribe(
            &mouse_sensitivity_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let sens = slider_to_sensitivity(value.start());
                    println!(
                        "DEBUG: Slider changed to: {} (sensitivity: {})",
                        value.start(),
                        sens
                    );
                    this.current_sensitivity = sens;
                    cx.notify();
                }
            },
        )
        .detach();

        Self {
            mouse_sensitivity_slider,
            current_sensitivity: current_sens,
            force_no_accel_checked: accel_setting,
        }
    }
}

impl Render for MouseSettings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let slider_raw = self.mouse_sensitivity_slider.read(cx).value().start();
        let current_sens = slider_to_sensitivity(slider_raw);
        let accel_setting = self.force_no_accel_checked;

        section_container(cx)
            .min_h(px(200.0))
            .child(
                with_tooltip(
                    "Mouse sensitivity ranges from -1.0 (slow) to 1.0 (fast). Acceleration adds variable speed based on movement - disable it for consistent 1:1 cursor tracking.",
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_color(cx.theme().foreground)
                        .child("Mouse settings".to_string()),
                    cx,
                )
            )
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        div()
                            .h_flex()
                            .gap_4()
                            .items_center()
                            .child(div().min_w(px(120.0)).child("Sensitivity:"))
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{:.2}", current_sens)),
                            ),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .min_w(px(120.0))
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Slow"),
                            )
                            .child(
                                Slider::new(&self.mouse_sensitivity_slider)
                                    .w_full()
                                    .text_color(cx.theme().foreground),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Fast"),
                            ),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child("Acceleration".to_string())
                    .child(
                        Switch::new("force-no-accel-switch")
                            .checked(self.force_no_accel_checked)
                            .on_click(cx.listener(|view, checked, _, cx| {
                                view.force_no_accel_checked = *checked;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)))
                    .child(
                        Button::new("apply-mouse-settings")
                            .label("Apply mouse setting")
                            .on_click(move |_, _, _cx| {
                                let sens = mouse_sensitivity_override(current_sens);
                                let force_no_accel = mouse_force_no_accel_override(accel_setting);

                                write_override_line(&sens).unwrap();
                                write_override_line(&force_no_accel).unwrap();
                            }),
                    ),
            )
    }
}
