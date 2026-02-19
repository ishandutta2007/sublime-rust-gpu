use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{init, Root};
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use std::env;
use std::fs; // New
use serde_json;

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
    current_dir: PathBuf,
    expanded_projects: HashSet<PathBuf>,
    char_widths: HashMap<char, f32>, // New field
    active_menu_button_bounds: Option<Bounds<f32>>,
}

// Constants for menu button sizing
const MENU_BUTTON_HORIZONTAL_PADDING_PX: f32 = 24.0; // Corresponds to px_3() (12px left + 12px right)
const MENU_BUTTON_CORRECTION_PX: f32 = 1.0; // Adjustment for visual alignment

impl AppView {
    fn new(_cx: &mut Context<Self>) -> Self {
        let charlen_json_content = fs::read_to_string("charlen_12px.json")
            .expect("Failed to read charlen_12px.json");
        let char_widths: HashMap<char, f32> = serde_json::from_str(&charlen_json_content)
            .expect("Failed to parse charlen_12px.json");

        Self {
            open_menu: OpenMenu::None,
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            expanded_projects: HashSet::new(),
            active_menu_button_bounds: None,
            char_widths, // Initialize with parsed data
        }
    }





    /// Recursively renders the project explorer tree.
    fn render_project_explorer(&self, path: PathBuf, cx: &mut Context<Self>) -> impl IntoElement {
        let is_expanded = self.expanded_projects.contains(&path);
        let project_name = path.file_name()
            .map_or("?", |os_str| os_str.to_str().unwrap_or("?"))
            .to_string();

        let project_label = div()
            .flex()
            .items_center()
            .child(if is_expanded { "▼ " } else { "▶ " })
            .child(project_name)
            .text_color(rgb(0xdddddd))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener({
                let path_clone = path.clone();
                move |_this, _, _, cx| {
                    if _this.expanded_projects.contains(&path_clone) {
                        _this.expanded_projects.remove(&path_clone);
                    } else {
                        _this.expanded_projects.insert(path_clone.clone());
                    }
                    cx.notify();
                }
            }));

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
                    let os_string = entry.file_name();
                    let file_name = os_string.to_str()
                        .unwrap_or("?")
                        .to_string();

                    if entry_path.is_dir() {
                        children_elements.push(self.render_project_explorer(entry_path.clone(), cx).into_any_element());
                    } else {
                        // File entry
                        children_elements.push(
                            div()
                                .px(px(16.0)) // Indent files
                                .child(file_name)
                                .text_color(rgb(0xaaaaaa))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, cx.listener({
                                    let entry_path_clone = entry_path.clone();
                                    move |_this, _, _, cx| {
                                        // TODO: Open file
                                        eprintln!("Clicked file: {:?}", entry_path_clone);
                                        cx.notify();
                                    }
                                }))
                                .into_any_element()
                        );
                    }
                }
            }
        }

        div()
            .flex()
            .flex_col()
            .child(project_label)
            .children(children_elements)
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Compute horizontal offset of the open menu button so the dropdown
        // can be absolutely positioned from the root — above the sidebar.
        let menu_bar_labels: &[(&str, OpenMenu)] = &[
            ("File",        OpenMenu::File),
            ("Edit",        OpenMenu::Edit),
            ("Selection",   OpenMenu::Selection),
            ("Find",        OpenMenu::Find),
            ("View",        OpenMenu::View),
            ("Goto",        OpenMenu::Goto),
            ("Tools",       OpenMenu::Tools),
            ("Project",     OpenMenu::Project),
            ("Preferences", OpenMenu::Preferences),
            ("Help",        OpenMenu::Help),
        ];

        // Approximate pixel width of each menu label button (px_3 = 12px padding + ~7px/char)
        let btn_width = |label: &str| {
            label.chars().map(|c| self.char_widths.get(&c).unwrap_or(&7.0)).sum::<f32>() + MENU_BUTTON_HORIZONTAL_PADDING_PX - MENU_BUTTON_CORRECTION_PX
        };

        let mut dropdown_left = 0.0f32;
        for (label, variant) in menu_bar_labels.iter() {
            if variant == &self.open_menu {
                break;
            }
            dropdown_left += btn_width(label);
        }

        let menu_bar_h = 26.0f32;

        div()
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .bg(rgb(0x232323))
            // ── Menu bar (labels only — no dropdowns nested here) ─────────
            .child(
                div()
                    .flex()
                    .flex_row()
                    .bg(rgb(0x1e1e1e))
                    .w_full()
                    .children(menu_bar_labels.iter().map(|(label, variant)| {
                        let is_open = variant == &self.open_menu;
                        let variant = variant.clone();
                        div()
                            .px_3()
                            .py_1()
                            .text_size(px(12.0))
                            .text_color(if is_open { rgb(0xffffff) } else { rgb(0xcccccc) })
                            .bg(if is_open { rgb(0x3e3e3e) } else { rgb(0x1e1e1e) })
                            .hover(|s| s.bg(rgb(0x3e3e3e)).text_color(rgb(0xcccccc)))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(
                                move |_this, _, _, cx| {
                                    _this.open_menu = if _this.open_menu == variant {
                                        OpenMenu::None
                                    } else {
                                        variant.clone()
                                    };
                                    cx.notify();
                                }
                            ))
                            .child(*label)
                            .into_any_element()
                    }))
            )
            // ── Sidebar + editor ──────────────────────────────────────────
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w(px(200.0))
                            .bg(rgb(0x1e1e1e))
                            .border_r_1()
                            .border_color(rgb(0x454545))
                            .p(px(8.0))
                            .text_color(rgb(0xcccccc))
                            .child(self.render_project_explorer(self.current_dir.clone(), cx))
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .justify_center()
                            .items_center()
                            .text_xl()
                            .text_color(rgb(0x555555))
                            .child("Hello, Sublime-rust!"),
                    )
            )
            // ── Dropdown overlay — rendered LAST so it paints on top ──────
            .when(self.open_menu != OpenMenu::None, |el: Div| {
                let items = match &self.open_menu {
                    OpenMenu::File        => file_menu_items(),
                    OpenMenu::Edit        => edit_menu_items(),
                    OpenMenu::Selection   => selection_menu_items(),
                    OpenMenu::Find        => find_menu_items(),
                    OpenMenu::View        => view_menu_items(),
                    OpenMenu::Goto        => goto_menu_items(),
                    OpenMenu::Tools       => tools_menu_items(),
                    OpenMenu::Project     => project_menu_items(),
                    OpenMenu::Preferences => preferences_menu_items(),
                    OpenMenu::Help        => help_menu_items(),
                    OpenMenu::None        => vec![],
                };
                el
                    // Full-window transparent capture layer — click outside to close
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .left(px(0.0))
                            .size_full()
                            .on_mouse_down(MouseButton::Left, cx.listener(|_this, _, _, cx| {
                                _this.open_menu = OpenMenu::None;
                                cx.notify();
                            }))
                    )
                    // The dropdown panel itself, anchored below the clicked button
                    .child(
                        div()
                            .absolute()
                            .top(px(menu_bar_h))
                            .left(px(dropdown_left))
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
                                        .when(item.has_arrow, |el| {
                                            el.child(
                                                div()
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(0x888888))
                                                    .child("▶"),
                                            )
                                        })
                                        .when_some(item.shortcut, |el, sc| {
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
                    )
            })
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
