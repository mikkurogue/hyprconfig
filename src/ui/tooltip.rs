use gpui::*;
use gpui_component::tooltip::Tooltip;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A helper function to add native GPUI tooltip to any element.
///
/// The tooltip appears on hover using the native GPUI tooltip system.
/// Note: Has a 500ms delay before appearing (GPUI default).
///
/// # Example Usage
///
/// ```rust
/// use crate::ui::tooltip::with_tooltip;
///
/// with_tooltip(
///     "This is a helpful tooltip",
///     div().child("Hover over me!"),
///     cx
/// )
/// ```
pub fn with_tooltip<E: IntoElement + 'static>(
    content: impl Into<SharedString>,
    child: E,
    _cx: &mut App,
) -> Stateful<Div> {
    let tooltip_text: SharedString = content.into();

    // Create a unique ID based on the tooltip content
    let mut hasher = DefaultHasher::new();
    tooltip_text.hash(&mut hasher);
    let unique_id = hasher.finish();

    div()
        .child(child)
        .id(("tooltip", unique_id))
        .tooltip(move |window, cx| Tooltip::new(tooltip_text.clone()).build(window, cx))
}
