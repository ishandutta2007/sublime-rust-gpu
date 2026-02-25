use gpui::*;
use gpui_component::{init, Root, v_flex, h_flex};
use gpui_component::scroll::{ScrollbarShow, ScrollableElement};
use gpui_component::theme::{Theme, ThemeMode};

struct ScrollDemo {
    left_handle: ScrollHandle,
    right_handle: ScrollHandle,
}

impl ScrollDemo {
    fn new() -> Self {
        Self {
            left_handle: ScrollHandle::new(),
            right_handle: ScrollHandle::new(),
        }
    }
}

impl Render for ScrollDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .bg(rgb(0x181818))
            .child(
                // Left Pane
                div()
                    .id("left-pane-wrapper")
                    .relative()
                    .flex_1()
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(0x333333))
                    .child(
                        v_flex()
                            .id("left-scroll-area")
                            .size_full()
                            .track_scroll(&self.left_handle)
                            .overflow_y_scroll()
                            .children((0..100).map(|i| {
                                div()
                                    .h(px(40.0))
                                    .px_4()
                                    .flex()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(rgb(0x222222))
                                    .text_color(rgb(0xaaaaaa))
                                    .child(format!("Left Item {}", i))
                            }))
                    )
                    .vertical_scrollbar(&self.left_handle)
            )
            .child(
                // Right Pane
                div()
                    .id("right-pane-wrapper")
                    .relative()
                    .flex_1()
                    .h_full()
                    .child(
                        v_flex()
                            .id("right-scroll-area")
                            .size_full()
                            .track_scroll(&self.right_handle)
                            .overflow_y_scroll()
                            .children((0..100).map(|i| {
                                div()
                                    .h(px(40.0))
                                    .px_4()
                                    .flex()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(rgb(0x222222))
                                    .text_color(rgb(0xcccccc))
                                    .child(format!("Right Item {}", i))
                            }))
                    )
                    .vertical_scrollbar(&self.right_handle)
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        init(cx);

        // Force high-contrast theme for scrollbars and fix click color change
        Theme::change(ThemeMode::Dark, None, cx);
        let theme = cx.global_mut::<Theme>();
        theme.scrollbar_show = ScrollbarShow::Always;
        theme.scrollbar_thumb = rgb(0xffffff).into();       // Solid White thumb
        theme.scrollbar_thumb_hover = rgb(0xffffff).into(); // Keep white on hover/click
        theme.scrollbar = rgb(0x444444).into();             // Dark Gray track

        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|_| ScrollDemo::new());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open window");
    });
}
