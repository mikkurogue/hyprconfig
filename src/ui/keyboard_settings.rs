use gpui::*;
use gpui_component::button::Button;
use gpui_component::dropdown::*;
use std::collections::HashSet;

use gpui_component::StyledExt;

use crate::{
    conf::{self, write_override_line},
    util::keyboard::{LocaleInfo, current_device_locales, sys_locales},
};

pub struct KeyboardSettings {
    selected_locales: HashSet<String>,
    available_locales: Vec<LocaleInfo>,
    locale_dropdown: Entity<DropdownState<Vec<String>>>,
}

impl KeyboardSettings {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        // Load available locales from XKB
        let available_locales = sys_locales().unwrap_or_else(|e| {
            eprintln!("Failed to load locales from XKB: {}, using fallback", e);
            vec![
                LocaleInfo {
                    code: "us".to_string(),
                    label: "English (US)".to_string(),
                },
                LocaleInfo {
                    code: "gb".to_string(),
                    label: "English (UK)".to_string(),
                },
                LocaleInfo {
                    code: "fi".to_string(),
                    label: "Finnish".to_string(),
                },
            ]
        });

        // Create labels for dropdown (display label with code)
        let locale_labels: Vec<String> = available_locales
            .iter()
            .map(|l| format!("{} ({})", l.label, l.code))
            .collect();

        let selected_locales = current_device_locales().unwrap_or_else(|e| {
            eprintln!("Failed to get current locales: {}, using default", e);
            let mut default_set = HashSet::new();
            default_set.insert("us".to_string());
            default_set
        });

        // Set initial dropdown selection to first locale in the set
        let current_locale_idx = selected_locales
            .iter()
            .next()
            .and_then(|locale| available_locales.iter().position(|l| &l.code == locale));

        let locale_dropdown = cx.new(|cx| {
            DropdownState::new(
                locale_labels.clone(),
                current_locale_idx.map(gpui_component::IndexPath::new),
                window,
                cx,
            )
        });

        // Subscribe to dropdown selection events
        cx.subscribe(
            &locale_dropdown,
            |this, _dropdown, event: &DropdownEvent<Vec<String>>, cx| {
                if let DropdownEvent::Confirm(Some(selected_label)) = event {
                    // Extract the code from the label format "Label (code)"
                    if let Some(code) = this.extract_code_from_label(selected_label) {
                        this.selected_locales.insert(code);
                        cx.notify();
                    }
                }
            },
        )
        .detach();

        KeyboardSettings {
            selected_locales,
            available_locales,
            locale_dropdown,
        }
    }

    fn remove_locale(&mut self, locale: &str, cx: &mut Context<Self>) {
        self.selected_locales.remove(locale);
        cx.notify();
    }

    fn extract_code_from_label(&self, label: &str) -> Option<String> {
        // Extract code from "Label (code)" format
        label
            .rfind('(')
            .and_then(|start| label.rfind(')').map(|end| (start, end)))
            .map(|(start, end)| label[start + 1..end].trim().to_string())
    }

    fn get_label_for_code(&self, code: &str) -> String {
        self.available_locales
            .iter()
            .find(|l| l.code == code)
            .map(|l| l.label.clone())
            .unwrap_or_else(|| code.to_string())
    }
}

impl Render for KeyboardSettings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_locales = self.selected_locales.clone();

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
                    .child("Input locales".to_string()),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)).child("Locale:"))
                    .child(Dropdown::new(&self.locale_dropdown).min_w(px(200.0))),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)).child("Selected:"))
                    .child(
                        div().h_flex().gap_2().flex_wrap().children(
                            self.selected_locales
                                .iter()
                                .enumerate()
                                .map(|(idx, locale)| {
                                    let locale_clone = locale.clone();
                                    let label = self.get_label_for_code(locale);
                                    div()
                                        .h_flex()
                                        .gap_1()
                                        .px_2()
                                        .py_1()
                                        .border_1()
                                        .border_color(rgb(0x606060))
                                        .rounded_md()
                                        .bg(rgb(0x2a2a2a))
                                        .items_center()
                                        .child(
                                            div()
                                                .text_sm()
                                                .child(format!("{} ({})", label, locale)),
                                        )
                                        .child(Button::new(("remove", idx)).label("×").on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                this.remove_locale(&locale_clone, cx);
                                            }),
                                        ))
                                }),
                        ),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .items_center()
                    .child(div().min_w(px(120.0)))
                    .child(
                        Button::new("apply-keyboard-settings")
                            .label("Apply keyboard config")
                            .on_click(move |_, _, _cx| {
                                // TODO: remove the clone here this is just dirty hack to get it
                                // working
                                let override_str = conf::locale_override(selected_locales.clone());

                                // DEBUG
                                println!("Generated locale override string: {}", override_str);

                                write_override_line(&override_str).unwrap_or_else(|e| {
                                    println!("Failed to write override line: {}", e);
                                });
                            }),
                    ),
            )
    }
}
