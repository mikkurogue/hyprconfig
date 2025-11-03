use gpui::*;
use gpui_component::button::Button;
use gpui_component::dropdown::*;

use gpui_component::StyledExt;

pub struct KeyboardSettings {
    locales: Vec<String>,
    locale_dropdown: Entity<DropdownState<Vec<String>>>,
}

impl KeyboardSettings {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        // example locales for now
        let locales = vec![
            "us".to_string(),
            "fb".to_string(),
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

        KeyboardSettings {
            locales,
            locale_dropdown,
        }
    }
}

impl Render for KeyboardSettings {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(div().min_w(px(120.0)).child("Resolution:"))
                    .child(Dropdown::new(&self.locale_dropdown).min_w(px(200.0))),
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
                                println!("Apply keyboard settings clicked ");
                            }),
                    ),
            )
    }
}
