use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::{ScrollableElement, ScrollbarShow};
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{h_flex, init, v_flex, Root};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;

actions!(sublime_rust, [Quit, Save, SaveAs, SaveAll]);

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

struct MenuItem {
    label: &'static str,
    shortcut: Option<&'static str>,
    action: Box<dyn Action>,
    is_separator: bool,
    has_arrow: bool,
}

impl Clone for MenuItem {
    fn clone(&self) -> Self {
        Self {
            label: self.label,
            shortcut: self.shortcut,
            action: self.action.boxed_clone(),
            is_separator: self.is_separator,
            has_arrow: self.has_arrow,
        }
    }
}

impl MenuItem {
    fn item(label: &'static str, shortcut: Option<&'static str>, action: impl Action) -> Self {
        Self {
            label,
            shortcut,
            action: action.boxed_clone(),
            is_separator: false,
            has_arrow: false,
        }
    }
    fn sep() -> Self {
        Self {
            label: "",
            shortcut: None,
            action: Quit.boxed_clone(),
            is_separator: true,
            has_arrow: false,
        }
    }
    fn submenu(label: &'static str) -> Self {
        Self {
            label,
            shortcut: None,
            action: Quit.boxed_clone(),
            is_separator: false,
            has_arrow: true,
        }
    }
}

fn file_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("New File", Some("Ctrl+N"), Save),
        MenuItem::sep(),
        MenuItem::item("Open File...", Some("Ctrl+O"), Save),
        MenuItem::item("Open Folder...", None, Save),
        MenuItem::submenu("Open Recent"),
        MenuItem::sep(),
        MenuItem::item("Reopen Closed File", None, Save),
        MenuItem::item("New View into File", None, Save),
        MenuItem::sep(),
        MenuItem::item("Save", Some("Ctrl+S"), Save),
        MenuItem::item("Save As...", Some("Ctrl+Shift+S"), SaveAs),
        MenuItem::item("Save All", Some("Ctrl+Alt+S"), SaveAll),
        MenuItem::sep(),
        MenuItem::item("Reload from Disk", None, Save),
        MenuItem::sep(),
        MenuItem::item("Close View", Some("Ctrl+W"), Quit),
        MenuItem::item("Close File", None, Quit),
        MenuItem::sep(),
        if cfg!(target_os = "macos") {
            MenuItem::item("Quit", Some("Cmd+Q"), Quit)
        } else {
            MenuItem::item("Exit", Some("Alt+F4"), Quit)
        },
    ]
}

fn edit_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Undo", Some("Ctrl+Z"), Save),
        MenuItem::item("Redo", Some("Ctrl+Y"), Save),
        MenuItem::sep(),
        MenuItem::item("Copy", Some("Ctrl+C"), Save),
        MenuItem::item("Cut", Some("Ctrl+X"), Save),
        MenuItem::item("Paste", Some("Ctrl+V"), Save),
        MenuItem::sep(),
        MenuItem::submenu("Line"),
        MenuItem::submenu("Comment"),
        MenuItem::submenu("Text"),
        MenuItem::submenu("Tag"),
    ]
}

fn selection_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Select All", Some("Ctrl+A"), Save),
        MenuItem::item("Expand Selection", Some("Ctrl+L"), Save),
        MenuItem::sep(),
        MenuItem::item("Add Next Line", Some("Ctrl+Alt+Down"), Save),
        MenuItem::item("Add Previous Line", Some("Ctrl+Alt+Up"), Save),
    ]
}

fn find_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Find...", Some("Ctrl+F"), Save),
        MenuItem::item("Find Next", Some("F3"), Save),
        MenuItem::item("Find Previous", Some("Shift+F3"), Save),
        MenuItem::item("Replace...", Some("Ctrl+H"), Save),
        MenuItem::sep(),
        MenuItem::item("Find in Files...", Some("Ctrl+Shift+F"), Save),
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
        MenuItem::item("Goto Anything...", Some("Ctrl+P"), Save),
        MenuItem::sep(),
        MenuItem::item("Goto Symbol...", Some("Ctrl+R"), Save),
        MenuItem::item("Goto Line...", Some("Ctrl+G"), Save),
    ]
}

fn tools_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Command Palette...", Some("Ctrl+Shift+P"), Save),
        MenuItem::sep(),
        MenuItem::submenu("Build System"),
        MenuItem::item("Build", Some("Ctrl+B"), Save),
    ]
}

fn project_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Open Project...", None, Save),
        MenuItem::submenu("Recent Projects"),
        MenuItem::sep(),
        MenuItem::item("Save Project As...", None, Save),
    ]
}

fn preferences_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Settings", None, Save),
        MenuItem::item("Key Bindings", None, Save),
        MenuItem::sep(),
        MenuItem::submenu("Color Scheme"),
        MenuItem::submenu("Theme"),
    ]
}

fn help_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Documentation", None, Save),
        MenuItem::item("Twitter", None, Save),
        MenuItem::sep(),
        MenuItem::item("About Sublime Text", None, Save),
    ]
}

struct ScrollDemo {
    left_handle: ScrollHandle,
    right_handle: ScrollHandle,
    focus_handle: FocusHandle,
    current_dir: PathBuf,
    expanded_dirs: HashSet<PathBuf>,
    open_tabs: Vec<PathBuf>,
    active_tab_index: Option<usize>,
    tab_contents: HashMap<PathBuf, Vec<String>>,
    dirty_tabs: HashSet<PathBuf>,
    open_menu: OpenMenu,
    sidebar_width: f32,
    is_dragging_sidebar: bool,
    cursor_row: usize,
    cursor_col: usize,

    // Syntect state
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_syntax_name: String,

    // Char widths
    char_widths: HashMap<char, f32>,
}

impl ScrollDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        // Load char widths from parent directory or current directory
        let char_widths = if let Ok(content) = fs::read_to_string("../charlen_arial_12px.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else if let Ok(content) = fs::read_to_string("charlen_arial_12px.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            left_handle: ScrollHandle::new(),
            right_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            expanded_dirs: HashSet::new(),
            open_tabs: Vec::new(),
            active_tab_index: None,
            tab_contents: HashMap::new(),
            dirty_tabs: HashSet::new(),
            open_menu: OpenMenu::None,
            sidebar_width: 250.0,
            is_dragging_sidebar: false,
            cursor_row: 0,
            cursor_col: 0,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_syntax_name: "Plain Text".to_string(),
            char_widths,
        }
    }

    fn update_syntax(&mut self) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx) {
                if let Some(syntax) = self.syntax_set.find_syntax_for_file(path).unwrap_or(None) {
                    self.current_syntax_name = syntax.name.clone();
                    return;
                }
            }
        }
        self.current_syntax_name = "Plain Text".to_string();
    }

    fn save_active(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                if let Some(lines) = self.tab_contents.get(&path) {
                    let content = lines.join("\n");
                    if fs::write(&path, content).is_ok() {
                        self.dirty_tabs.remove(&path);
                        eprintln!("Saved: {:?}", path);
                        cx.notify();
                    }
                }
            }
        }
    }

    fn save_as(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                if let Some(lines) = self.tab_contents.get(&path) {
                    let mut new_path = path.clone();
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
                    new_path.set_file_name(format!("{}_copy.txt", stem));
                    let content = lines.join("\n");
                    if fs::write(&new_path, content).is_ok() {
                        eprintln!("Saved As: {:?}", new_path);
                        cx.notify();
                    }
                }
            }
        }
    }

    fn save_all(&mut self, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = self.dirty_tabs.iter().cloned().collect();
        for path in paths {
            if let Some(lines) = self.tab_contents.get(&path) {
                let content = lines.join("\n");
                if fs::write(&path, content).is_ok() {
                    self.dirty_tabs.remove(&path);
                    eprintln!("Saved (All): {:?}", path);
                }
            }
        }
        cx.notify();
    }

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
                        children_elements.push(
                            div()
                                .pl(px(16.0))
                                .child(file_name)
                                .text_color(rgb(0xaaaaaa))
                                .hover(|s| s.bg(rgb(0x2d2d2d)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener({
                                        let entry_path_clone = entry_path.clone();
                                        move |this, _, window, cx| {
                                            if let Some(pos) = this.open_tabs.iter().position(|p| p == &entry_path_clone) {
                                                this.active_tab_index = Some(pos);
                                            } else {
                                                if let Ok(content) = fs::read_to_string(&entry_path_clone) {
                                                    let lines = content.lines().map(|s| s.to_string()).collect();
                                                    this.tab_contents.insert(entry_path_clone.clone(), lines);
                                                    this.open_tabs.push(entry_path_clone.clone());
                                                    this.active_tab_index = Some(this.open_tabs.len() - 1);
                                                }
                                            }
                                            this.update_syntax();
                                            this.cursor_row = 0;
                                            this.cursor_col = 0;
                                            this.right_handle.set_offset(Point::default());
                                            window.focus(&this.focus_handle);
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
                        .pl(px(12.0))
                        .children(children_elements),
                )
            })
    }
}

impl Render for ScrollDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = _window.focused(cx) == Some(self.focus_handle.clone());
        let active_lines = self.active_tab_index
            .and_then(|idx| self.open_tabs.get(idx))
            .and_then(|path| self.tab_contents.get(path))
            .cloned()
            .unwrap_or_else(|| vec!["Click a file in the explorer to see its content here.".to_string()]);

        let syntax = self.active_tab_index
            .and_then(|idx| self.open_tabs.get(idx))
            .and_then(|path| self.syntax_set.find_syntax_for_file(path).ok().flatten())
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

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
        let footer_h = 22.0f32;

        div()
            .id("root")
            .relative()
            .size_full()
            .bg(rgb(0x181818))
            .on_action(cx.listener(|this, _action: &Save, _window, cx| this.save_active(cx)))
            .on_action(cx.listener(|this, _action: &SaveAs, _window, cx| this.save_as(cx)))
            .on_action(cx.listener(|this, _action: &SaveAll, _window, cx| this.save_all(cx)))
            .on_action(cx.listener(|_this, _action: &Quit, _window, cx| cx.quit()))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if this.is_dragging_sidebar {
                    this.sidebar_width = event.position.x.into();
                    this.sidebar_width = this.sidebar_width.clamp(50.0, 600.0);
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
                            .text_color(if is_open { rgb(0xffffff) } else { rgb(0xcccccc) })
                            .bg(if is_open { rgb(0x3e3e3e) } else { rgb(0x1e1e1e) })
                            .hover(|s| s.bg(rgb(0x3e3e3e)).text_color(rgb(0xcccccc)))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    _this.open_menu = if _this.open_menu == variant { OpenMenu::None } else { variant.clone() };
                                    cx.notify();
                                }),
                            )
                            .child(*label)
                    })),
            )
            .child(
                div()
                    .absolute()
                    .top(px(menu_bar_h))
                    .bottom(px(footer_h))
                    .left_0()
                    .right_0()
                    .child(
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
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                this.is_dragging_sidebar = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("right-pane-wrapper")
                            .absolute()
                            .top_0()
                            .left(px(self.sidebar_width + 5.0))
                            .right_0()
                            .bottom_0()
                            .child(
                                h_flex()
                                    .bg(rgb(0x1e1e1e))
                                    .h(px(30.0))
                                    .overflow_x_hidden()
                                    .children(self.open_tabs.iter().enumerate().map(|(idx, path)| {
                                        let is_active = Some(idx) == self.active_tab_index;
                                        let mut file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                                        if self.dirty_tabs.contains(path) { file_name.push('*'); }
                                        let path_clone = path.clone();
                                        div()
                                            .flex().items_center().px(px(10.0)).h_full()
                                            .bg(if is_active { rgb(0x232323) } else { rgb(0x181818) })
                                            .border_r_1().border_color(rgb(0x333333))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                this.active_tab_index = Some(idx);
                                                this.update_syntax();
                                                this.right_handle.set_offset(Point::default());
                                                window.focus(&this.focus_handle);
                                                cx.notify();
                                            }))
                                            .child(div().text_size(px(12.0)).text_color(if is_active { rgb(0xcccccc) } else { rgb(0x888888) }).child(file_name))
                                            .child(
                                                div().ml(px(8.0)).text_size(px(10.0)).text_color(rgb(0x666666)).hover(|s| s.text_color(rgb(0xcccccc)))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                        this.open_tabs.remove(idx);
                                                        this.tab_contents.remove(&path_clone);
                                                        this.dirty_tabs.remove(&path_clone);
                                                        if let Some(active_idx) = this.active_tab_index {
                                                            if active_idx >= this.open_tabs.len() {
                                                                this.active_tab_index = if this.open_tabs.is_empty() { None } else { Some(this.open_tabs.len() - 1) };
                                                            }
                                                        }
                                                        this.update_syntax();
                                                        cx.stop_propagation();
                                                        cx.notify();
                                                    }))
                                                    .child("✕")
                                            )
                                    })),
                            )
                            .child(
                                div()
                                    .id("editor-area-wrapper")
                                    .absolute()
                                    .top(px(30.0))
                                    .bottom_0()
                                    .left_0()
                                    .right_0()
                                    .border_1()
                                    .border_color(if is_focused { rgb(0x094771) } else { rgb(0x333333) })
                                    .track_focus(&self.focus_handle)
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                        window.focus(&this.focus_handle);
                                        cx.notify();
                                    }))
                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                                        if event.keystroke.modifiers.platform || event.keystroke.modifiers.control { return; }
                                        if let Some(idx) = this.active_tab_index {
                                            let path = this.open_tabs[idx].clone();
                                            if let Some(lines) = this.tab_contents.get_mut(&path) {
                                                match event.keystroke.key.as_str() {
                                                    "backspace" => {
                                                        if this.cursor_col > 0 {
                                                            lines[this.cursor_row].remove(this.cursor_col - 1);
                                                            this.cursor_col -= 1;
                                                            this.dirty_tabs.insert(path);
                                                        } else if this.cursor_row > 0 {
                                                            let current_line = lines.remove(this.cursor_row);
                                                            this.cursor_row -= 1;
                                                            this.cursor_col = lines[this.cursor_row].len();
                                                            lines[this.cursor_row].push_str(&current_line);
                                                            this.dirty_tabs.insert(path);
                                                        }
                                                    }
                                                    "enter" => {
                                                        let current_line = &mut lines[this.cursor_row];
                                                        let new_line = current_line.split_off(this.cursor_col);
                                                        lines.insert(this.cursor_row + 1, new_line);
                                                        this.cursor_row += 1;
                                                        this.cursor_col = 0;
                                                        this.dirty_tabs.insert(path);
                                                    }
                                                    "left" => {
                                                        if this.cursor_col > 0 { this.cursor_col -= 1; }
                                                        else if this.cursor_row > 0 {
                                                            this.cursor_row -= 1;
                                                            this.cursor_col = lines[this.cursor_row].len();
                                                        }
                                                    }
                                                    "right" => {
                                                        if this.cursor_col < lines[this.cursor_row].len() { this.cursor_col += 1; }
                                                        else if this.cursor_row < lines.len() - 1 {
                                                            this.cursor_row += 1;
                                                            this.cursor_col = 0;
                                                        }
                                                    }
                                                    "up" => {
                                                        if this.cursor_row > 0 {
                                                            this.cursor_row -= 1;
                                                            this.cursor_col = this.cursor_col.min(lines[this.cursor_row].len());
                                                        }
                                                    }
                                                    "down" => {
                                                        if this.cursor_row < lines.len() - 1 {
                                                            this.cursor_row += 1;
                                                            this.cursor_col = this.cursor_col.min(lines[this.cursor_row].len());
                                                        }
                                                    }
                                                    "space" => { lines[this.cursor_row].insert(this.cursor_col, ' '); this.cursor_col += 1; this.dirty_tabs.insert(path); }
                                                    "tab" => { lines[this.cursor_row].insert_str(this.cursor_col, "    "); this.cursor_col += 4; this.dirty_tabs.insert(path); }
                                                    key if key.len() == 1 => {
                                                        lines[this.cursor_row].insert_str(this.cursor_col, key);
                                                        this.cursor_col += 1;
                                                        this.dirty_tabs.insert(path);
                                                    }
                                                    _ => {}
                                                }
                                                cx.notify();
                                            }
                                        }
                                    }))
                                    .child(
                                        v_flex()
                                            .id("right-scroll-area")
                                            .size_full()
                                            .track_scroll(&self.right_handle)
                                            .overflow_y_scroll()
                                            .child(
                                                v_flex().flex_none().p(px(16.0)).children(
                                                    active_lines.into_iter().enumerate().map(|(i, line)| {
                                                        let is_cursor_row = is_focused && Some(i) == Some(self.cursor_row);
                                                        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(&line, &self.syntax_set).unwrap_or_default();
                                                        let mut span_elements: Vec<AnyElement> = vec![];
                                                        let mut current_offset = 0;

                                                        for (style, text) in ranges {
                                                            let color = style.foreground;
                                                            let text_len = text.len();
                                                            if is_cursor_row && self.cursor_col >= current_offset && self.cursor_col < current_offset + text_len {
                                                                let split_idx = self.cursor_col - current_offset;
                                                                let (before, after) = text.split_at(split_idx);
                                                                if !before.is_empty() { span_elements.push(div().text_color(rgb(u32::from_be_bytes([0, color.r, color.g, color.b]))).child(before.to_string()).into_any_element()); }
                                                                span_elements.push(div().text_color(rgb(0xffffff)).child("|").into_any_element());
                                                                if !after.is_empty() { span_elements.push(div().text_color(rgb(u32::from_be_bytes([0, color.r, color.g, color.b]))).child(after.to_string()).into_any_element()); }
                                                            } else {
                                                                span_elements.push(div().text_color(rgb(u32::from_be_bytes([0, color.r, color.g, color.b]))).child(text.to_string()).into_any_element());
                                                            }
                                                            current_offset += text_len;
                                                        }
                                                        if is_cursor_row && self.cursor_col >= current_offset { span_elements.push(div().text_color(rgb(0xffffff)).child("|").into_any_element()); }

                                                        h_flex()
                                                            .id(i)
                                                            .flex_none()
                                                            .h(px(20.0))
                                                            .font_family("Courier New")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                this.cursor_row = i;
                                                                if let Some(tab_idx) = this.active_tab_index {
                                                                    let path = &this.open_tabs[tab_idx];
                                                                    this.cursor_col = this.tab_contents.get(path).map(|l| l[i].len()).unwrap_or(0);
                                                                }
                                                                cx.notify();
                                                            }))
                                                            .child(
                                                                div()
                                                                    .w(px(40.0))
                                                                    .text_color(rgb(0x666666))
                                                                    .text_size(px(12.0))
                                                                    .flex()
                                                                    .justify_end()
                                                                    .pr(px(8.0))
                                                                    .child(format!("{}", i + 1))
                                                            )
                                                            .children(span_elements)
                                                    }),
                                                ),
                                            ),
                                    )
                                    .vertical_scrollbar(&self.right_handle)
                            )
                    )
            )
            .child(
                h_flex()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(footer_h))
                    .bg(rgb(0x007acc))
                    .px_4()
                    .justify_between()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().text_size(px(11.0)).text_color(rgb(0xffffff)).child(format!("Ln {}, Col {}", self.cursor_row + 1, self.cursor_col + 1)))
                    )
                    .child(
                        div().text_size(px(11.0)).text_color(rgb(0xffffff)).child(self.current_syntax_name.clone())
                    )
            )
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
                    _ => vec![] 
                };
                
                // Helper to calculate label width precisely
                let get_label_width = |label: &str, char_widths: &HashMap<char, f32>| {
                    label.chars()
                        .map(|c| char_widths.get(&c).unwrap_or(&7.0))
                        .sum::<f32>() + 24.0
                };

                let mut dropdown_left = 0.0f32;
                for (label, variant) in menu_bar_labels.iter() {
                    if variant == &self.open_menu { break; }
                    dropdown_left += get_label_width(label, &self.char_widths);
                }

                el.child(div().absolute().top_0().left_0().size_full().on_mouse_down(MouseButton::Left, cx.listener(|_this, _, _, cx| { _this.open_menu = OpenMenu::None; cx.notify(); })))
                  .child(v_flex().absolute().top(px(menu_bar_h)).left(px(dropdown_left)).w(px(270.0)).bg(rgb(0x2d2d2d)).border_1().border_color(rgb(0x454545)).shadow_lg().py(px(4.0)).children(items.into_iter().map(|item| {
                      if item.is_separator { div().h(px(1.0)).my(px(3.0)).mx(px(8.0)).bg(rgb(0x444444)).into_any_element() }
                      else {
                          let action = item.action.boxed_clone();
                          let label = item.label;
                          h_flex().id(label).justify_between().items_center().px(px(12.0)).py(px(3.0)).text_size(px(12.0)).text_color(rgb(0xcccccc)).hover(|s| s.bg(rgb(0x094771)).text_color(rgb(0xffffff))).cursor_pointer()
                                  .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| { 
                                      let action_ref = action.as_ref();
                                      if label == "Save" { this.save_active(cx); }
                                      else if label == "Save All" { this.save_all(cx); }
                                      else if label == "Save As..." { this.save_as(cx); }
                                      else if label == "Quit" || label == "Exit" { cx.quit(); }
                                      else { cx.dispatch_action(action_ref); }
                                      this.open_menu = OpenMenu::None;
                                      cx.notify();
                                  }))
                                  .child(item.label)
                                  .when(item.has_arrow, |el| el.child(div().text_size(px(10.0)).text_color(rgb(0x888888)).child("▶")))
                                  .when_some(item.shortcut, |el, sc| el.child(div().text_size(px(10.0)).text_color(rgb(0x888888)).child(sc)))
                                  .into_any_element()
                      }
                  })))
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        init(cx);
        
        cx.bind_keys([
            KeyBinding::new("cmd-s", Save, None),
            KeyBinding::new("ctrl-s", Save, None),
            KeyBinding::new("cmd-shift-s", SaveAs, None),
            KeyBinding::new("ctrl-shift-s", SaveAs, None),
            KeyBinding::new("cmd-alt-s", SaveAll, None),
            KeyBinding::new("ctrl-alt-s", SaveAll, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);

        Theme::change(ThemeMode::Dark, None, cx);
        let theme = cx.global_mut::<Theme>();
        theme.scrollbar_show = ScrollbarShow::Always;
        theme.scrollbar_thumb = rgb(0xffffff).into(); 
        theme.scrollbar_thumb_hover = rgb(0xffffff).into(); 
        theme.scrollbar = rgb(0x2a2a2a).into(); 
        let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
        cx.open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, |window, cx| {
            let view = cx.new(|cx| ScrollDemo::new(cx));
            cx.new(|cx| Root::new(view, window, cx))
        }).expect("failed to open window");
    });
}
