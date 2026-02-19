use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{init, Root};

actions!(sublime_rust, [Quit]);

// ── Menu state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum OpenMenu {
    None,
    File,
}

// ── File-menu dropdown items ──────────────────────────────────────────────────

#[derive(Clone)]
struct MenuItem {
    label: &'static str,
    shortcut: Option<&'static str>,
    is_separator: bool,
    has_arrow: bool,
}

impl MenuItem {
    fn item(label: &'static str, shortcut: Option<&'static str>) -> Self {
        Self { label, shortcut, is_separator: false, has_arrow: false }
    }
    fn sep() -> Self {
        Self { label: "", shortcut: None, is_separator: true, has_arrow: false }
    }
    fn submenu(label: &'static str) -> Self {
        Self { label, shortcut: None, is_separator: false, has_arrow: true }
    }
}

fn file_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("New File",           Some("Ctrl+N")),
        MenuItem::sep(),
        MenuItem::item("Open File...",        Some("Ctrl+O")),
        MenuItem::item("Open Folder...",      None),
        MenuItem::submenu("Open Recent"),
        MenuItem::sep(),
        MenuItem::item("Reopen Closed File",  None),
        MenuItem::item("New View into File",  None),
        MenuItem::sep(),
        MenuItem::item("Save",                Some("Ctrl+S")),
        MenuItem::item("Save As...",          None),
        MenuItem::item("Save All",            None),
        MenuItem::sep(),
        MenuItem::item("Reload from Disk",    None),
        MenuItem::sep(),
        MenuItem::item("Close View",          Some("Ctrl+W")),
        MenuItem::item("Close File",          None),
        MenuItem::sep(),
        if cfg!(target_os = "macos") {
            MenuItem::item("Quit", Some("Cmd+Q"))
        } else {
            MenuItem::item("Exit", Some("Alt+F4"))
        },
    ]
}

// ── App view ──────────────────────────────────────────────────────────────────

struct AppView {
    open_menu: OpenMenu,
}

impl AppView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self { open_menu: OpenMenu::None }
    }

    /// Render the dropdown panel for the File menu
    fn render_file_dropdown(&self) -> impl IntoElement {
        div()
            .absolute()
            .top(px(26.0))
            .left(px(0.0))
            .w(px(270.0))
            .bg(rgb(0x2d2d2d))
            .border_1()
            .border_color(rgb(0x454545))
            .shadow_lg()
            .py(px(4.0))
            .children(file_menu_items().into_iter().map(|item| {
                if item.is_separator {
                    div()
                        .h(px(1.0))
                        .my(px(3.0))
                        .mx(px(8.0))
                        .bg(rgb(0x444444))
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .px(px(12.0))
                        .py(px(3.0))
                        .text_size(px(12.0))
                        .text_color(rgb(0xcccccc))
                        .hover(|s| s.bg(rgb(0x094771)).text_color(rgb(0xffffff)))
                        .cursor_pointer()
                        .child(item.label)
                        .when(item.has_arrow, |el: Div| {
                            el.child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(rgb(0x888888))
                                    .child("▶"),
                            )
                        })
                        .when_some(item.shortcut, |el: Div, sc| {
                            el.child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(rgb(0x888888))
                                    .child(sc),
                            )
                        })
                        .into_any_element()
                }
            }))
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file_open = self.open_menu == OpenMenu::File;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x232323))
            // ── Menu bar ──────────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .flex_row()
                    .bg(rgb(0x1e1e1e))
                    .w_full()
                    // File button — uses gpui-component Button which is confirmed working
                    .child(
                        div()
                            .relative()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .text_size(px(12.0))
                                    .text_color(rgb(0xcccccc))
                                    .hover(|s| s.bg(rgb(0x3e3e3e)).text_color(rgb(0xcccccc)))
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.open_menu = if this.open_menu == OpenMenu::File {
                                            OpenMenu::None
                                        } else {
                                            OpenMenu::File
                                        };
                                        cx.notify();
                                    }))
                                    .child("File"),
                            )
                            .when(file_open, |el: Div| el.child(self.render_file_dropdown())),
                    )
                    // Other menu bar labels (static for now)
                    .child(plain_menu_label("Edit"))
                    .child(plain_menu_label("Selection"))
                    .child(plain_menu_label("Find"))
                    .child(plain_menu_label("View"))
                    .child(plain_menu_label("Goto"))
                    .child(plain_menu_label("Tools"))
                    .child(plain_menu_label("Project"))
                    .child(plain_menu_label("Preferences"))
                    .child(plain_menu_label("Help")),
            )
            // ── Main content ──────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .flex_1()
                    .justify_center()
                    .items_center()
                    .text_xl()
                    .text_color(rgb(0x555555))
                    .child("Hello, Sublime-rust!"),
            )
    }
}

fn plain_menu_label(label: &'static str) -> impl IntoElement {
    div()
        .px_3()
        .py_1()
        .text_size(px(12.0))
        .text_color(rgb(0xcccccc))
        .hover(|s| s.bg(rgb(0x3e3e3e)))
        .cursor_pointer()
        .child(label)
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    Application::new().run(|cx: &mut App| {
        // Required by gpui-component
        init(cx);

        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| AppView::new(cx));
                // Root is required by gpui-component for event routing to work
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open window");
    });
}
