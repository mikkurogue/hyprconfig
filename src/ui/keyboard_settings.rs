use gpui::*;
use gpui_component::button::Button;
use gpui_component::dropdown::*;
use std::collections::HashSet;

use gpui_component::{ActiveTheme as _, StyledExt};

use crate::{
    conf::{self, write_override_line},
    ui::{item_pill::item_pill, section_container::section_container, tooltip::with_tooltip},
    util::keyboard::{
        Keyboard, LocaleInfo, current_device_locales, get_all_keyboards, sys_locales,
    },
};

pub struct KeyboardSettings {
    selected_locales: HashSet<String>,
    available_locales: Vec<LocaleInfo>,
    devices: Vec<crate::util::keyboard::Keyboard>,
    device_dropdowns: Vec<Entity<DropdownState<Vec<String>>>>,
}

impl KeyboardSettings {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        let keyboards = get_all_keyboards().unwrap_or_else(|e| {
            eprintln!("Failed to get keyboards: {}", e);
            vec![]
        });

        println!("Detected keyboards: {:?}", keyboards);

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

        // Create a dropdown per device so each has its own state/id
        let mut device_dropdowns = Vec::new();
        for _device in keyboards.iter() {
            let dd = cx.new(|cx| {
                DropdownState::new(
                    locale_labels.clone(),
                    current_locale_idx.map(gpui_component::IndexPath::new),
                    window,
                    cx,
                )
            });
            device_dropdowns.push(dd);
        }

        // Subscribe to each dropdown selection
        for dd in device_dropdowns.iter() {
            cx.subscribe(
                dd,
                |this, _dropdown, event: &DropdownEvent<Vec<String>>, cx| {
                    if let DropdownEvent::Confirm(Some(selected_label)) = event {
                        if let Some(code) = this.extract_code_from_label(selected_label) {
                            this.selected_locales.insert(code);
                            cx.notify();
                        }
                    }
                },
            )
            .detach();
        }

        KeyboardSettings {
            selected_locales,
            available_locales,
            devices: keyboards,
            device_dropdowns,
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
        let devices = &self.devices;

        let devices_view = section_container(cx)
            .flex_col()
            .gap_3()
            .child(with_tooltip(
                "Detected the following keyboard input devices. Beware that some may be duplicates and only 1 of them needs to have a setup",
                div().font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child("Keyboards".to_string()),
                cx,
            ))
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .flex_wrap()
                    .children(devices.iter().enumerate().map(|(idx, d)| {
                        let dropdown = &self.device_dropdowns[idx];
                        div()
                            .flex_col()
                            .gap_2()
                            .p_6()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(div().font_weight(FontWeight::BOLD).child(d.name.clone()))
                            .child(div().text_sm().child(format!("Current layout: {}", d.layout)))
                            .child(div().h_flex().child(Dropdown::new(dropdown).min_w(px(200.0))))
                            .child(
                                Button::new(("apply-keyboard-settings", idx))
                                    .label("Apply keyboard config")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let device_name = this.devices[idx].name.clone();
                                        let dropdown_state = this.device_dropdowns[idx].read(cx);
                                        if let Some(sel) = dropdown_state.selected_index(cx) {
                                            let locale_code = this.available_locales[sel.row].code.clone();
                                            let override_str = conf::set_keyboard_device_layout(device_name.clone(), locale_code.clone());
                                            println!("Generated override string: {}", override_str);
                                            write_override_line(&override_str).unwrap_or_else(|e| println!("Failed to write override line: {}", e));
                                        } else {
                                            println!("No locale selected for {}", device_name);
                                        }
                                    })),
                            )
                    }))
            );

        devices_view

        // section_container(cx)
        //     .min_h(px(200.0))
        //     .child(
        //         with_tooltip(
        //             "Keyboard locales determine your keyboard layout. You can select multiple locales and switch between them. The first locale in the list will be your default.",
        //             div()
        //                 .font_weight(FontWeight::BOLD)
        //                 .text_color(cx.theme().foreground)
        //                 .child("Input locales".to_string()),
        //             cx,
        //         )
        //     )
        //     .child(
        //         div()
        //             .h_flex()
        //             .gap_4()
        //             .items_center()
        //             .child(div().min_w(px(120.0)).child("Locale:"))
        //             .child(/* old single dropdown removed */)
        //     )
        //     .child(
        //         div()
        //             .h_flex()
        //             .gap_4()
        //             .items_center()
        //             .child(div().min_w(px(120.0)).child("Selected:"))
        //             .child(
        //                 div().h_flex().gap_2().flex_wrap().children(
        //                     self.selected_locales
        //                         .iter()
        //                         .enumerate()
        //                         .map(|(idx, locale)| {
        //                             let locale_clone = locale.clone();
        //                             let label = self.get_label_for_code(locale);
        //                             item_pill(cx)
        //                                 .child(
        //                                     div()
        //                                         .text_sm()
        //                                         .child(format!("{} ({})", label, locale)),
        //                                 )
        //                                 .child(Button::new(("remove", idx)).label("×").on_click(
        //                                     cx.listener(move |this, _, _, cx| {
        //                                         this.remove_locale(&locale_clone, cx);
        //                                     }),
        //                                 ))
        //                         }),
        //                 ),
        //             ),
        //     )
        //     .child(
        //         div()
        //             .h_flex()
        //             .gap_4()
        // //             .items_center()
        //             .child(div().min_w(px(120.0)))
        // ,
        //     )
    }
}
