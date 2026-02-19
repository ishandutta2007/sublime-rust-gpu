use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{init, Root};

actions!(sublime_rust, [Quit]);

// ── Menu state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum OpenMenu {
    None,
    File,
    Edit,
    Selection,
    Find,
    View,
    Goto,
    Tools,
    Project,
    Preferences,
    Help,
}

// ── Dropdown items for all menus ──────────────────────────────────────────────

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

fn edit_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Undo", Some("Ctrl+Z")),
        MenuItem::item("Redo", Some("Ctrl+Y")),
        MenuItem::sep(),
        MenuItem::item("Copy", Some("Ctrl+C")),
        MenuItem::item("Cut", Some("Ctrl+X")),
        MenuItem::item("Paste", Some("Ctrl+V")),
        MenuItem::sep(),
        MenuItem::submenu("Line"),
        MenuItem::submenu("Comment"),
        MenuItem::submenu("Text"),
        MenuItem::submenu("Tag"),
    ]
}

fn selection_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Select All", Some("Ctrl+A")),
        MenuItem::item("Expand Selection", Some("Ctrl+L")),
        MenuItem::sep(),
        MenuItem::item("Add Next Line", Some("Ctrl+Alt+Down")),
        MenuItem::item("Add Previous Line", Some("Ctrl+Alt+Up")),
    ]
}

fn find_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Find...", Some("Ctrl+F")),
        MenuItem::item("Find Next", Some("F3")),
        MenuItem::item("Find Previous", Some("Shift+F3")),
        MenuItem::item("Replace...", Some("Ctrl+H")),
        MenuItem::sep(),
        MenuItem::item("Find in Files...", Some("Ctrl+Shift+F")),
    ]
}

fn view_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::submenu("Side Bar"),
        MenuItem::submenu("Show Console"),
        MenuItem::sep(),
        MenuItem::submenu("Layout"),
        MenuItem::submenu("Groups"),
    ]
}

fn goto_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Goto Anything...", Some("Ctrl+P")),
        MenuItem::sep(),
        MenuItem::item("Goto Symbol...", Some("Ctrl+R")),
        MenuItem::item("Goto Line...", Some("Ctrl+G")),
    ]
}

fn tools_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Command Palette...", Some("Ctrl+Shift+P")),
        MenuItem::sep(),
        MenuItem::submenu("Build System"),
        MenuItem::item("Build", Some("Ctrl+B")),
    ]
}

fn project_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Open Project...", None),
        MenuItem::submenu("Recent Projects"),
        MenuItem::sep(),
        MenuItem::item("Save Project As...", None),
    ]
}

fn preferences_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Settings", None),
        MenuItem::item("Key Bindings", None),
        MenuItem::sep(),
        MenuItem::submenu("Color Scheme"),
        MenuItem::submenu("Theme"),
    ]
}

fn help_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Documentation", None),
        MenuItem::item("Twitter", None),
        MenuItem::sep(),
        MenuItem::item("About Sublime Text", None),
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

    /// Render a dropdown panel for a given menu
    fn render_dropdown(&self, items: Vec<MenuItem>) -> impl IntoElement {
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
            .children(items.into_iter().map(|item| {
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

    /// Render a single menu bar button and its associated dropdown
    fn render_menu_button(
        &self,
        label: &'static str,
        menu: OpenMenu,
        items: Vec<MenuItem>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_open = self.open_menu == menu;

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
                    .on_mouse_down(MouseButton::Left, cx.listener({
                        let menu = menu.clone();
                        move |this, _, _, cx| {
                            if this.open_menu == menu {
                                this.open_menu = OpenMenu::None;
                            } else {
                                this.open_menu = menu.clone();
                            }
                            cx.notify();
                        }
                    }))
                    .child(label),
            )
            .when(is_open, |el: Div| el.child(self.render_dropdown(items)))
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(self.render_menu_button("File", OpenMenu::File, file_menu_items(), cx))
                    .child(self.render_menu_button("Edit", OpenMenu::Edit, edit_menu_items(), cx))
                    .child(self.render_menu_button("Selection", OpenMenu::Selection, selection_menu_items(), cx))
                    .child(self.render_menu_button("Find", OpenMenu::Find, find_menu_items(), cx))
                    .child(self.render_menu_button("View", OpenMenu::View, view_menu_items(), cx))
                    .child(self.render_menu_button("Goto", OpenMenu::Goto, goto_menu_items(), cx))
                    .child(self.render_menu_button("Tools", OpenMenu::Tools, tools_menu_items(), cx))
                    .child(self.render_menu_button("Project", OpenMenu::Project, project_menu_items(), cx))
                    .child(self.render_menu_button("Preferences", OpenMenu::Preferences, preferences_menu_items(), cx))
                    .child(self.render_menu_button("Help", OpenMenu::Help, help_menu_items(), cx)),
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
