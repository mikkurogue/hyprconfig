use gpui::*;

use crate::ui::keyboard_settings::KeyboardSettings;
use crate::ui::mouse_settings::MouseSettings;
use crate::ui::section_container::section_sub_container;

// Grouped input settings (keyboard now, mouse later)
pub struct InputSettings {
    keyboard_settings: Entity<KeyboardSettings>,
    mouse_settings: Entity<MouseSettings>,
}

impl InputSettings {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let keyboard_settings = cx.new(|cx| KeyboardSettings::new(window, cx));
        let mouse_settings = cx.new(|cx| MouseSettings::new(window, cx));
        Self {
            keyboard_settings,
            mouse_settings,
        }
    }
}

impl Render for InputSettings {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        section_sub_container()
            .child(self.keyboard_settings.clone())
            .child(self.mouse_settings.clone())
    }
}
