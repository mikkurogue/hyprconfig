use gpui::*;
use gpui_component::button::Button;
use gpui_component::dropdown::*;
use std::collections::HashSet;

use gpui_component::StyledExt;

pub struct KeyboardSettings {
    locales: Vec<String>,
    selected_locales: HashSet<String>,
    locale_dropdown: Entity<DropdownState<Vec<String>>>,
}

impl KeyboardSettings {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        // example locales for now
        let locales = vec![
            "us".to_string(),
            "gb".to_string(),
            "fi".to_string(),
            "dk".to_string(),
            "no".to_string(),
            "de".to_string(),
        ];

        let current_locale_idx = locales.iter().position(|l| l == "fi");

        let locale_dropdown = cx.new(|cx| {
            DropdownState::new(
                locales.clone(),
                current_locale_idx.map(gpui_component::IndexPath::new),
                window,
                cx,
            )
        });

        let mut selected_locales = HashSet::new();
        if let Some(idx) = current_locale_idx {
            selected_locales.insert(locales[idx].clone());
        }

        // Subscribe to dropdown selection events
        cx.subscribe(
            &locale_dropdown,
            |this, _dropdown, event: &DropdownEvent<Vec<String>>, cx| {
                if let DropdownEvent::Confirm(Some(selected_value)) = event {
                    // HashSet automatically handles uniqueness
                    this.selected_locales.insert(selected_value.clone());
                    cx.notify();
                }
            },
        )
        .detach();

        KeyboardSettings {
            locales,
            selected_locales,
            locale_dropdown,
        }
    }

    fn remove_locale(&mut self, locale: &str, cx: &mut Context<Self>) {
        self.selected_locales.remove(locale);
        cx.notify();
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
                    .text_lg()
                    .child(format!("Keyboard Settings")),
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
                                        .child(div().text_sm().child(locale.clone()))
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
                                println!(
                                    "Apply keyboard settings clicked with locales: {:?}",
                                    selected_locales
                                );
                            }),
                    ),
            )
    }
}
