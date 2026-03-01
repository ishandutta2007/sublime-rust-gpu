use gpui::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use ignore::WalkBuilder;
use crate::menu::OpenMenu;

pub struct ScrollDemo {
    pub left_handle: ScrollHandle,
    pub right_handle: ScrollHandle,
    pub focus_handle: FocusHandle,
    pub find_focus_handle: FocusHandle,
    pub fif_focus_find: FocusHandle,
    pub fif_focus_where: FocusHandle,
    pub fif_focus_replace: FocusHandle,
    pub current_dir: PathBuf,
    pub expanded_dirs: HashSet<PathBuf>,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab_index: Option<usize>,
    pub tab_contents: HashMap<PathBuf, Vec<String>>,
    pub dirty_tabs: HashSet<PathBuf>,
    pub open_menu: OpenMenu,
    pub sidebar_width: f32,
    pub is_dragging_sidebar: bool,
    pub cursor_row: usize,
    pub cursor_col: usize,

    // Syntect state
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub current_syntax_name: String,

    // Char widths
    pub char_widths: HashMap<char, f32>,

    // Confirmation dialog state
    pub pending_close_path: Option<PathBuf>,

    // Find state
    pub find_active: bool,
    pub find_query: String,
    pub find_matches: Vec<(usize, usize)>, // (row, col)
    pub active_match_index: Option<usize>,

    // Find in Files state
    pub fif_active: bool,
    pub fif_query: String,
    pub fif_where: String,
    pub fif_replace: String,
    pub fif_use_gitignore: bool,
}

impl ScrollDemo {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
            find_focus_handle: cx.focus_handle(),
            fif_focus_find: cx.focus_handle(),
            fif_focus_where: cx.focus_handle(),
            fif_focus_replace: cx.focus_handle(),
            fif_where: current_dir.to_string_lossy().to_string(),
            current_dir,
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
            pending_close_path: None,
            find_active: false,
            find_query: String::new(),
            find_matches: Vec::new(),
            active_match_index: None,
            fif_active: false,
            fif_query: String::new(),
            fif_replace: String::new(),
            fif_use_gitignore: true,
        }
    }

    pub fn update_syntax(&mut self) {
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

    pub fn save_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(lines) = self.tab_contents.get(&path) {
            let content = lines.join("
");
            if fs::write(&path, content).is_ok() {
                self.dirty_tabs.remove(&path);
                eprintln!("Saved: {:?}", path);
                cx.notify();
            }
        }
    }

    pub fn save_active(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                self.save_path(path, cx);
            }
        }
    }

    pub fn save_as(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                if let Some(lines) = self.tab_contents.get(&path) {
                    let mut new_path = path.clone();
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
                    new_path.set_file_name(format!("{}_copy.txt", stem));
                    let content = lines.join("
");
                    if fs::write(&new_path, content).is_ok() {
                        eprintln!("Saved As: {:?}", new_path);
                        cx.notify();
                    }
                }
            }
        }
    }

    pub fn save_all(&mut self, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = self.dirty_tabs.iter().cloned().collect();
        for path in paths {
            if let Some(lines) = self.tab_contents.get(&path) {
                let content = lines.join("
");
                if fs::write(&path, content).is_ok() {
                    self.dirty_tabs.remove(&path);
                    eprintln!("Saved (All): {:?}", path);
                }
            }
        }
        cx.notify();
    }

    pub fn perform_search(&mut self) {
        self.find_matches.clear();
        if self.find_query.is_empty() {
            self.active_match_index = None;
            return;
        }

        if let Some(idx) = self.active_tab_index {
            let path = &self.open_tabs[idx];
            if let Some(lines) = self.tab_contents.get(path) {
                for (row, line) in lines.iter().enumerate() {
                    let mut start = 0;
                    while let Some(pos) = line[start..].find(&self.find_query) {
                        self.find_matches.push((row, start + pos));
                        start += pos + self.find_query.len().max(1);
                    }
                }
            }
        }

        if !self.find_matches.is_empty() {
            self.active_match_index = Some(0);
            self.jump_to_active_match();
        } else {
            self.active_match_index = None;
        }
    }

    pub fn jump_to_active_match(&mut self) {
        if let Some(idx) = self.active_match_index {
            let (row, col) = self.find_matches[idx];
            self.cursor_row = row;
            self.cursor_col = col;

            let line_height = 20.0;
            let top_padding = 16.0;
            let target_y = (row as f32 * line_height) + top_padding - 100.0;
            self.right_handle.set_offset(Point::new(px(0.0), px(-target_y.max(0.0))));
        }
    }

    pub fn find_next(&mut self, cx: &mut Context<Self>) {
        if !self.find_matches.is_empty() {
            let next = self.active_match_index.map(|i| (i + 1) % self.find_matches.len()).unwrap_or(0);
            self.active_match_index = Some(next);
            self.jump_to_active_match();
            cx.notify();
        }
    }

    pub fn find_prev(&mut self, cx: &mut Context<Self>) {
        if !self.find_matches.is_empty() {
            let prev = self.active_match_index.map(|i| if i == 0 { self.find_matches.len() - 1 } else { i - 1 }).unwrap_or(0);
            self.active_match_index = Some(prev);
            self.jump_to_active_match();
            cx.notify();
        }
    }

    pub fn perform_find_in_files(&mut self, cx: &mut Context<Self>) {
        if self.fif_query.is_empty() { return; }
        let search_path = PathBuf::from(&self.fif_where);
        if !search_path.exists() { return; }

        let mut results = Vec::new();
        let mut file_count = 0;
        let mut match_count = 0;

        let walk = if self.fif_use_gitignore {
            WalkBuilder::new(&search_path).build()
        } else {
            ignore::Walk::new(&search_path)
        };

        for entry in walk {
            if let Ok(entry) = entry {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        file_count += 1;
                        let lines: Vec<_> = content.lines().collect();
                        let mut file_matches = Vec::new();
                        for (i, line) in lines.iter().enumerate() {
                            if line.contains(&self.fif_query) {
                                file_matches.push(format!("  {:>4}: {}", i + 1, line));
                                match_count += 1;
                            }
                        }
                        if !file_matches.is_empty() {
                            results.push(format!("{}:", entry.path().display()));
                            results.extend(file_matches);
                            results.push(String::new());
                        }
                    }
                }
            }
        }

        let mut final_content = vec![
            format!("Searching {} files for term \"{}\"", file_count, self.fif_query),
            String::new(),
        ];
        final_content.extend(results.into_iter());
        final_content.push(format!("{} matches found in {} files", match_count, file_count));

        let results_path = PathBuf::from("Find Results");
        self.tab_contents.insert(results_path.clone(), final_content);
        if !self.open_tabs.contains(&results_path) {
            self.open_tabs.push(results_path.clone());
        }
        self.active_tab_index = self.open_tabs.iter().position(|p| p == &results_path);
        self.fif_active = false;
        cx.notify();
    }

    pub fn perform_replace_in_files(&mut self, cx: &mut Context<Self>) {
        if self.fif_query.is_empty() { return; }
        let search_path = PathBuf::from(&self.fif_where);
        if !search_path.exists() { return; }

        let walk = if self.fif_use_gitignore {
            WalkBuilder::new(&search_path).build()
        } else {
            ignore::Walk::new(&search_path)
        };

        for entry in walk {
            if let Ok(entry) = entry {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains(&self.fif_query) {
                            let new_content = content.replace(&self.fif_query, &self.fif_replace);
                            let _ = fs::write(entry.path(), new_content);
                        }
                    }
                }
            }
        }
        self.fif_active = false;
        cx.notify();
    }

    pub fn close_tab(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(idx) = self.open_tabs.iter().position(|p| p == &path) {
            self.open_tabs.remove(idx);
            self.tab_contents.remove(&path);
            self.dirty_tabs.remove(&path);
            if let Some(active_idx) = self.active_tab_index {
                if active_idx >= self.open_tabs.len() {
                    self.active_tab_index = if self.open_tabs.is_empty() { None } else { Some(self.open_tabs.len() - 1) };
                }
            }
            self.update_syntax();
            cx.notify();
        }
    }

    pub fn request_close_tab(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.dirty_tabs.contains(&path) {
            self.pending_close_path = Some(path);
        } else {
            self.close_tab(path, cx);
        }
        cx.notify();
    }
}
