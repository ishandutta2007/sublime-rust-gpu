use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::{ScrollableElement, ScrollbarShow};
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{h_flex, init, v_flex, Root};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

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

// ── MenuItem ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MenuItem {
    label: &'static str,
    shortcut: Option<&'static str>,
    is_separator: bool,
    has_arrow: bool,
}

impl MenuItem {
    fn item(label: &'static str, shortcut: Option<&'static str>) -> Self {
        Self {
            label,
            shortcut,
            is_separator: false,
            has_arrow: false,
        }
    }
    fn sep() -> Self {
        Self {
            label: "",
            shortcut: None,
            is_separator: true,
            has_arrow: false,
        }
    }
    fn submenu(label: &'static str) -> Self {
        Self {
            label,
            shortcut: None,
            is_separator: false,
            has_arrow: true,
        }
    }
}

fn file_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("New File", Some("Ctrl+N")),
        MenuItem::sep(),
        MenuItem::item("Open File...", Some("Ctrl+O")),
        MenuItem::item("Open Folder...", None),
        MenuItem::submenu("Open Recent"),
        MenuItem::sep(),
        MenuItem::item("Reopen Closed File", None),
        MenuItem::item("New View into File", None),
        MenuItem::sep(),
        MenuItem::item("Save", Some("Ctrl+S")),
        MenuItem::item("Save As...", None),
        MenuItem::item("Save All", None),
        MenuItem::sep(),
        MenuItem::item("Reload from Disk", None),
        MenuItem::sep(),
        MenuItem::item("Close View", Some("Ctrl+W")),
        MenuItem::item("Close File", None),
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

struct ScrollDemo {
    left_handle: ScrollHandle,
    right_handle: ScrollHandle,
    current_dir: PathBuf,
    expanded_dirs: HashSet<PathBuf>,
    open_tabs: Vec<PathBuf>,
    active_tab_index: Option<usize>,
    tab_contents: HashMap<PathBuf, String>,
    open_menu: OpenMenu,
    sidebar_width: f32,
    is_dragging_sidebar: bool,
}

impl ScrollDemo {
    fn new() -> Self {
        Self {
            left_handle: ScrollHandle::new(),
            right_handle: ScrollHandle::new(),
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            expanded_dirs: HashSet::new(),
            open_tabs: Vec::new(),
            active_tab_index: None,
            tab_contents: HashMap::new(),
            open_menu: OpenMenu::None,
            sidebar_width: 250.0,
            is_dragging_sidebar: false,
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
                        let _entry_path_clone = entry_path.clone();
                        children_elements.push(
                            div()
                                .pl(px(16.0)) // Align with directory text
                                .child(file_name)
                                .text_color(rgb(0xaaaaaa))
                                .hover(|s| s.bg(rgb(0x2d2d2d)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener({
                                        let _entry_path_clone = entry_path.clone();
                                        move |_this, _, _, cx| {
                                            if let Some(pos) = _this.open_tabs.iter().position(|p| p == &_entry_path_clone) {
                                                _this.active_tab_index = Some(pos);
                                            } else {
                                                if let Ok(content) = fs::read_to_string(&_entry_path_clone) {
                                                    _this.tab_contents.insert(_entry_path_clone.clone(), content);
                                                    _this.open_tabs.push(_entry_path_clone.clone());
                                                    _this.active_tab_index = Some(_this.open_tabs.len() - 1);
                                                }
                                            }
                                            _this.right_handle.set_offset(Point::default());
                                            cx.stop_propagation();
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
        let active_content = self.active_tab_index
            .and_then(|idx| self.open_tabs.get(idx))
            .and_then(|path| self.tab_contents.get(path))
            .map(|s| s.as_str())
            .unwrap_or("Click a file in the explorer to see its content here.");

        let menu_bar_labels: &[(&str, OpenMenu)] = &[
            ("File", OpenMenu::File),
            ("Edit", OpenMenu::Edit),
            ("Selection", OpenMenu::Selection),
            ("Find", OpenMenu::Find),
            ("View", OpenMenu::View),
            ("Goto", OpenMenu::Goto),
            ("Tools", OpenMenu::Tools),
            ("Project", OpenMenu::Project),
            ("Preferences", OpenMenu::Preferences),
            ("Help", OpenMenu::Help),
        ];

        let menu_bar_h = 26.0f32;

        div()
            .id("root")
            .relative()
            .size_full()
            .bg(rgb(0x181818))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if this.is_dragging_sidebar {
                    this.sidebar_width = event.position.x.into();
                    if this.sidebar_width < 50.0 {
                        this.sidebar_width = 50.0;
                    }
                    if this.sidebar_width > 600.0 {
                        this.sidebar_width = 600.0;
                    }
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _window, cx| {
                    if this.is_dragging_sidebar {
                        this.is_dragging_sidebar = false;
                        cx.notify();
                    }
                }),
            )
            // ── Menu Bar ──────────────────────────────────────────────────
            .child(
                h_flex()
                    .bg(rgb(0x1e1e1e))
                    .w_full()
                    .h(px(menu_bar_h))
                    .children(menu_bar_labels.iter().map(|(label, variant)| {
                        let is_open = variant == &self.open_menu;
                        let variant = variant.clone();
                        div()
                            .px_3()
                            .py_1()
                            .text_size(px(12.0))
                            .text_color(if is_open {
                                rgb(0xffffff)
                            } else {
                                rgb(0xcccccc)
                            })
                            .bg(if is_open {
                                rgb(0x3e3e3e)
                            } else {
                                rgb(0x1e1e1e)
                            })
                            .hover(|s| s.bg(rgb(0x3e3e3e)).text_color(rgb(0xcccccc)))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    _this.open_menu = if _this.open_menu == variant {
                                        OpenMenu::None
                                    } else {
                                        variant.clone()
                                    };
                                    cx.notify();
                                }),
                            )
                            .child(*label)
                    })),
            )
            // ── Main Content Area ─────────────────────────────────────────
            .child(
                div()
                    .absolute()
                    .top(px(menu_bar_h))
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .child(
                        // ── Left Pane (Project Explorer) ─────────────────────────────
                        div()
                            .id("left-pane-wrapper")
                            .absolute()
                            .top_0()
                            .left_0()
                            .bottom_0()
                            .w(px(self.sidebar_width))
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
                    // ── Separator ────────────────────────────────────────────────
                    .child(
                        div()
                            .id("separator")
                            .absolute()
                            .top_0()
                            .left(px(self.sidebar_width))
                            .w(px(5.0))
                            .bottom_0()
                            .bg(rgb(0x333333))
                            .cursor_col_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.is_dragging_sidebar = true;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        // ── Right Pane (Tabs + Code Editor) ──────────────────────────
                        div()
                            .id("right-pane-wrapper")
                            .absolute()
                            .top_0()
                            .left(px(self.sidebar_width + 2.0))
                            .right_0()
                            .bottom_0()
                            // ── Tab Bar ──────────────────────────────────────────
                            .child(
                                h_flex()
                                    .bg(rgb(0x1e1e1e))
                                    .h(px(30.0))
                                    .overflow_x_hidden()
                                    .children(self.open_tabs.iter().enumerate().map(|(idx, path)| {
                                        let is_active = Some(idx) == self.active_tab_index;
                                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                                        let path_clone = path.clone();
                                        
                                        div()
                                            .flex()
                                            .items_center()
                                            .px(px(10.0))
                                            .h_full()
                                            .bg(if is_active { rgb(0x232323) } else { rgb(0x181818) })
                                            .border_r_1()
                                            .border_color(rgb(0x333333))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.active_tab_index = Some(idx);
                                                this.right_handle.set_offset(Point::default());
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .text_color(if is_active { rgb(0xcccccc) } else { rgb(0x888888) })
                                                    .child(file_name)
                                            )
                                            .child(
                                                div()
                                                    .ml(px(8.0))
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(0x666666))
                                                    .hover(|s| s.text_color(rgb(0xcccccc)))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                        this.open_tabs.remove(idx);
                                                        this.tab_contents.remove(&path_clone);
                                                        if let Some(active_idx) = this.active_tab_index {
                                                            if active_idx >= this.open_tabs.len() {
                                                                this.active_tab_index = if this.open_tabs.is_empty() { None } else { Some(this.open_tabs.len() - 1) };
                                                            }
                                                        }
                                                        cx.stop_propagation();
                                                        cx.notify();
                                                    }))
                                                    .child("✕")
                                            )
                                    })),
                            )
                            // ── Editor Area ──────────────────────────────────────
                            .child(
                                div()
                                    .id("editor-area-wrapper")
                                    .absolute()
                                    .top(px(30.0))
                                    .bottom_0()
                                    .left_0()
                                    .right_0()
                                    .child(
                                        v_flex()
                                            .id("right-scroll-area")
                                            .size_full()
                                            .track_scroll(&self.right_handle)
                                            .overflow_y_scroll()
                                            .child(
                                                v_flex().flex_none().p(px(16.0)).children(
                                                    active_content.lines().enumerate().map(
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
                                    .vertical_scrollbar(&self.right_handle)
                            )
                    )
            )
            // ── Dropdown Overlays ─────────────────────────────────────────
            .when(self.open_menu != OpenMenu::None, |el| {
                let items = match &self.open_menu {
                    OpenMenu::File => file_menu_items(),
                    OpenMenu::Edit => edit_menu_items(),
                    OpenMenu::Selection => selection_menu_items(),
                    OpenMenu::Find => find_menu_items(),
                    OpenMenu::View => view_menu_items(),
                    OpenMenu::Goto => goto_menu_items(),
                    OpenMenu::Tools => tools_menu_items(),
                    OpenMenu::Project => project_menu_items(),
                    OpenMenu::Preferences => preferences_menu_items(),
                    OpenMenu::Help => help_menu_items(),
                    OpenMenu::None => vec![],
                };

                let mut dropdown_left = 0.0f32;
                let btn_width = |label: &str| label.len() as f32 * 8.0 + 24.0;
                
                for (label, variant) in menu_bar_labels.iter() {
                    if variant == &self.open_menu {
                        break;
                    }
                    dropdown_left += btn_width(label);
                }

                el
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _, _, cx| {
                                    _this.open_menu = OpenMenu::None;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        v_flex()
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
                                    h_flex()
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
                            })),
                    )
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        init(cx);

        // Force persistent white scrollbars
        Theme::change(ThemeMode::Dark, None, cx);
        let theme = cx.global_mut::<Theme>();
        theme.scrollbar_show = ScrollbarShow::Always;
        theme.scrollbar_thumb = rgb(0xffffff).into(); 
        theme.scrollbar_thumb_hover = rgb(0xffffff).into(); 
        theme.scrollbar = rgb(0x2a2a2a).into(); 

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
