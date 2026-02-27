use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::{ScrollableElement, ScrollbarShow};
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{h_flex, init, v_flex, Root};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

struct ScrollDemo {
    left_handle: ScrollHandle,
    right_handle: ScrollHandle,
    current_dir: PathBuf,
    expanded_dirs: HashSet<PathBuf>,
    active_file_content: String,
}

impl ScrollDemo {
    fn new() -> Self {
        Self {
            left_handle: ScrollHandle::new(),
            right_handle: ScrollHandle::new(),
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            expanded_dirs: HashSet::new(),
            active_file_content: "Click a file in the explorer to see its content here."
                .to_string(),
        }
    }

    /// Recursively renders the project explorer tree.
    fn render_project_explorer(&self, path: PathBuf, cx: &mut Context<Self>) -> impl IntoElement {
        let is_expanded = self.expanded_dirs.contains(&path);
        let dir_name = path
            .file_name()
            .map_or("?", |os_str| os_str.to_str().unwrap_or("?"))
            .to_string();

        let dir_label = div()
            .flex()
            .items_center()
            .child(
                div()
                    .w(px(12.0))
                    .flex()
                    .justify_center()
                    .child(if is_expanded { "▾" } else { "▸" }),
            )
            .child(div().pl(px(4.0)).child(dir_name))
            .text_color(rgb(0xdddddd))
            .hover(|s| s.bg(rgb(0x2d2d2d)))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener({
                    let path_clone = path.clone();
                    move |_this, _, _, cx| {
                        if _this.expanded_dirs.contains(&path_clone) {
                            _this.expanded_dirs.remove(&path_clone);
                        } else {
                            _this.expanded_dirs.insert(path_clone.clone());
                        }
                        cx.notify();
                    }
                }),
            );

        let mut children_elements: Vec<AnyElement> = vec![];
        if is_expanded {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut sorted_entries: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
                sorted_entries.sort_by(|a, b| {
                    let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for entry in sorted_entries {
                    let entry_path = entry.path();
                    let file_name = entry.file_name().to_str().unwrap_or("?").to_string();

                    if entry_path.is_dir() {
                        children_elements.push(
                            self.render_project_explorer(entry_path.clone(), cx)
                                .into_any_element(),
                        );
                    } else {
                        // File entry
                        let entry_path_clone = entry_path.clone();
                        children_elements.push(
                            div()
                                .pl(px(16.0)) // Align with directory text
                                .child(file_name)
                                .text_color(rgb(0xaaaaaa))
                                .hover(|s| s.bg(rgb(0x2d2d2d)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if let Ok(content) = fs::read_to_string(&entry_path_clone) {
                                            this.active_file_content = content;
                                            this.right_handle.set_offset(Point::default());
                                            cx.notify();
                                        }
                                    }),
                                )
                                .into_any_element(),
                        );
                    }
                }
            }
        }

        v_flex()
            .child(dir_label)
            .when(!children_elements.is_empty(), |el| {
                el.child(
                    v_flex()
                        .pl(px(12.0)) // Indent nested children
                        .children(children_elements),
                )
            })
    }
}

impl Render for ScrollDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Use absolute positioning for both panes so each has fully resolved pixel
        // bounds at paint time. flex_1() defers size resolution to after layout,
        // which means the scrollbar thumb has no height to paint into.
        // With absolute positioning GPUI resolves left/right/top/bottom into pixel
        // bounds before the paint pass runs, so the scrollbar renders correctly.
        div()
            .id("root")
            .relative()
            .size_full()
            .bg(rgb(0x181818))
            .child(
                // ── Left Pane (Project Explorer) ─────────────────────────────
                div()
                    .id("left-pane-wrapper")
                    .absolute()
                    .top_0()
                    .left_0()
                    .bottom_0()
                    .w(px(250.0))
                    .border_r_1()
                    .border_color(rgb(0x333333))
                    .child(
                        v_flex()
                            .id("left-scroll-area")
                            .size_full()
                            .track_scroll(&self.left_handle)
                            .overflow_y_scroll()
                            .child(
                                v_flex().flex_none().p_2().child(
                                    self.render_project_explorer(self.current_dir.clone(), cx),
                                ),
                            ),
                    )
                    .vertical_scrollbar(&self.left_handle),
            )
            .child(
                // ── Right Pane (Code Editor) ─────────────────────────────────
                // Also absolutely positioned: left edge is at 250px (after divider),
                // right/top/bottom stretch to fill. These are pixel-resolved before
                // the paint pass, so vertical_scrollbar() gets real bounds.
                div()
                    .id("right-pane-wrapper")
                    .absolute()
                    .top_0()
                    .left(px(251.0)) // 250px left pane + 1px border
                    .right_0()
                    .bottom_0()
                    .child(
                        v_flex()
                            .id("right-scroll-area")
                            .size_full()
                            .track_scroll(&self.right_handle)
                            .overflow_y_scroll()
                            .child(
                                v_flex().flex_none().p(px(16.0)).children(
                                    self.active_file_content.lines().enumerate().map(
                                        |(i, line)| {
                                            div()
                                                .id(i)
                                                .flex_none()
                                                .h(px(20.0))
                                                .text_color(rgb(0xcccccc))
                                                .font_family("Courier New")
                                                .child(line.to_string())
                                        },
                                    ),
                                ),
                            ),
                    )
                    .vertical_scrollbar(&self.right_handle),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        init(cx);

        // Force persistent white scrollbars
        Theme::change(ThemeMode::Dark, None, cx);
        let theme = cx.global_mut::<Theme>();
        theme.scrollbar_show = ScrollbarShow::Always;
        theme.scrollbar_thumb = rgb(0xffffff).into(); // Solid White thumb
        theme.scrollbar_thumb_hover = rgb(0xffffff).into(); // Keep white on hover/click
        theme.scrollbar = rgb(0x2a2a2a).into(); // Dark Gray track

        let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);

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
