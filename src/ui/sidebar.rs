use gpui::*;
use gpui_component::{
    Side, h_flex,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};

use crate::{ActiveSection, Hyprsetting};

pub fn create_sidebar(
    active_section: ActiveSection,
    cx: &mut Context<Hyprsetting>,
) -> impl IntoElement {
    Sidebar::new(Side::Left)
        .collapsed(false)
        .collapsible(true)
        .header(SidebarHeader::new().child(h_flex().child("Hypland settings")))
        .child(
            SidebarGroup::new("Displays").child(
                SidebarMenu::new().child(
                    SidebarMenuItem::new("Monitors")
                        .active(active_section == ActiveSection::Monitors)
                        .on_click(cx.listener(|view: &mut Hyprsetting, _, _, cx| {
                            view.set_active_section(ActiveSection::Monitors, cx);
                        })),
                ),
            ),
        )
        .child(
            SidebarGroup::new("Input").child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new("Keyboard")
                            .active(active_section == ActiveSection::Keyboard)
                            .on_click(cx.listener(|view: &mut Hyprsetting, _, _, cx| {
                                view.set_active_section(ActiveSection::Keyboard, cx);
                            })),
                    )
                    .child(
                        SidebarMenuItem::new("Mouse")
                            .active(active_section == ActiveSection::Mouse)
                            .on_click(cx.listener(|view: &mut Hyprsetting, _, _, cx| {
                                view.set_active_section(ActiveSection::Mouse, cx);
                            })),
                    ),
            ),
        )
}
