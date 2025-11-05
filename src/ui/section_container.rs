use gpui::*;
use gpui_component::{
    StyledExt,
    scroll::{Scrollable, ScrollbarAxis},
};

/// Common title style for sections
pub fn section_title(title_text: impl IntoElement) -> Div {
    div()
        .text_xl()
        .font_weight(FontWeight::BOLD)
        .child(title_text)
}

/// Common divider style for sections
pub fn section_divider() -> Div {
    div()
        .w_full()
        .h_1()
        .rounded_sm()
        .bg(hsla(198.0, 0.60, 0.92, 1.0))
}

/// Main container for the application and any subsequent modals/dialogs/popups/sub-windows
pub fn main_container() -> Scrollable<Div> {
    div()
        .v_flex()
        .gap_4()
        .size_full()
        .scrollable(ScrollbarAxis::Vertical)
        .p_4()
}

/// Common container style for sections
pub fn section_container() -> Div {
    div().v_flex().gap_2().p_4().border_1().rounded_sm()
}

/// Common sub-container style for sections
pub fn section_sub_container() -> Div {
    div()
        .h_flex()
        .gap_2()
        .p_3()
        .border_1()
        .border_color(Rgba {
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 0.3,
        })
        .rounded_sm()
}
