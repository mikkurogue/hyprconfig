use gpui::*;
use gpui_component::StyledExt;

pub fn item_pill() -> Div {
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
}
