use gpui::*;
use crate::actions::*;

#[derive(Clone, PartialEq)]
pub enum OpenMenu {
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

pub struct MenuItem {
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: Box<dyn Action>,
    pub is_separator: bool,
    pub has_arrow: bool,
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
    pub fn item(label: &'static str, shortcut: Option<&'static str>, action: impl Action) -> Self {
        Self {
            label,
            shortcut,
            action: action.boxed_clone(),
            is_separator: false,
            has_arrow: false,
        }
    }
    pub fn sep() -> Self {
        Self {
            label: "",
            shortcut: None,
            action: Quit.boxed_clone(),
            is_separator: true,
            has_arrow: false,
        }
    }
    pub fn submenu(label: &'static str) -> Self {
        Self {
            label,
            shortcut: None,
            action: Quit.boxed_clone(),
            is_separator: false,
            has_arrow: true,
        }
    }
}

pub fn file_menu_items() -> Vec<MenuItem> {
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

pub fn edit_menu_items() -> Vec<MenuItem> {
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

pub fn selection_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Select All", Some("Ctrl+A"), Save),
        MenuItem::item("Expand Selection", Some("Ctrl+L"), Save),
        MenuItem::sep(),
        MenuItem::item("Add Next Line", Some("Ctrl+Alt+Down"), Save),
        MenuItem::item("Add Previous Line", Some("Ctrl+Alt+Up"), Save),
    ]
}

pub fn find_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Find...", Some("Ctrl+F"), FindAction),
        MenuItem::item("Find Next", Some("F3"), Save),
        MenuItem::item("Find Previous", Some("Shift+F3"), Save),
        MenuItem::item("Replace...", Some("Ctrl+H"), Save),
        MenuItem::sep(),
        MenuItem::item("Find in Files...", Some("Ctrl+Shift+F"), FindInFilesAction),
    ]
}

pub fn view_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::submenu("Side Bar"),
        MenuItem::submenu("Show Console"),
        MenuItem::sep(),
        MenuItem::submenu("Layout"),
        MenuItem::submenu("Groups"),
    ]
}

pub fn goto_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Goto Anything...", Some("Ctrl+P"), Save),
        MenuItem::sep(),
        MenuItem::item("Goto Symbol...", Some("Ctrl+R"), Save),
        MenuItem::item("Goto Line...", Some("Ctrl+G"), Save),
    ]
}

pub fn tools_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Command Palette...", Some("Ctrl+Shift+P"), Save),
        MenuItem::sep(),
        MenuItem::submenu("Build System"),
        MenuItem::item("Build", Some("Ctrl+B"), Save),
    ]
}

pub fn project_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Open Project...", None, Save),
        MenuItem::submenu("Recent Projects"),
        MenuItem::sep(),
        MenuItem::item("Save Project As...", None, Save),
    ]
}

pub fn preferences_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Settings", None, Save),
        MenuItem::item("Key Bindings", None, Save),
        MenuItem::sep(),
        MenuItem::submenu("Color Scheme"),
        MenuItem::submenu("Theme"),
    ]
}

pub fn help_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Documentation", None, Save),
        MenuItem::item("Twitter", None, Save),
        MenuItem::sep(),
        MenuItem::item("About Sublime Text", None, Save),
    ]
}
