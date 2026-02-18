use gpui::*;

struct HelloWorld {
    // Save line numbers so you can calculate
    line_numbers: Vec<usize>,
}

impl HelloWorld {
    fn render_menu_item(&self, label: &'static str) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .text_size(px(12.0))
            .text_color(rgb(0xcccccc))
            .hover(|style| style.bg(rgb(0x3e3e3e)).text_color(rgb(0xffffff)))
            .child(label)
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x232323))
            .size_full()
            .child(
                // Menu Bar
                div()
                    .flex()
                    .bg(rgb(0x1e1e1e))
                    .w_full()
                    .child(self.render_menu_item("File"))
                    .child(self.render_menu_item("Edit"))
                    .child(self.render_menu_item("Selection"))
                    .child(self.render_menu_item("Find"))
                    .child(self.render_menu_item("View"))
                    .child(self.render_menu_item("Goto"))
                    .child(self.render_menu_item("Tools"))
                    .child(self.render_menu_item("Project"))
                    .child(self.render_menu_item("Preferences"))
                    .child(self.render_menu_item("Help")),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .bg(rgb(0x1e1e1e))
                            .w(px(40.0))
                            .pt(px(4.0))
                            .children(
                                self.line_numbers.iter().map(|n|
                                    div()
                                        .h(px(21.0)) 
                                        .text_color(rgb(0x666666))
                                        .text_size(px(12.0))
                                        .ml_2()
                                        .child(format!("{:>3}", n)) // Right Aligned
                                ).collect::<Vec<_>>() // // IMPORTANT: collect() to make it a Vec
                            )
                    )
                    .child(
                        // Editor area
                        div()
                            .flex_1()
                            .p_2()
                            .font_family("Monaco")
                            .text_size(px(14.0))
                            .line_height(px(21.0)) 
                            .text_color(rgb(0xcccccc))
                            .child("Hello, Sublime-rust!\nThis is line 2\nAnd line 3")
                    )
            )
    }
}

actions!(sublime_rust, [
    Quit,
    NewFile,
    Open,
    Save,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,
    GotoAnything,
    CommandPalette
]);

fn main() {
    Application::new().run(|cx| {
        cx.on_action(|_action: &Quit, cx| cx.quit());

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(100.0), px(100.0)),
                size: size(px(1024.0), px(768.0)),
            })),
            ..Default::default()
        };

        // Still set the system menus for OS-level integration (e.g. shortcuts)
        cx.set_menus(vec![
            Menu {
                name: "File".into(),
                items: vec![
                    MenuItem::action("New File", NewFile),
                    MenuItem::action("Open...", Open),
                    MenuItem::separator(),
                    MenuItem::action("Save", Save),
                    MenuItem::separator(),
                    MenuItem::action("Quit", Quit),
                ],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::action("Undo", Undo),
                    MenuItem::action("Redo", Redo),
                    MenuItem::separator(),
                    MenuItem::action("Cut", Cut),
                    MenuItem::action("Copy", Copy),
                    MenuItem::action("Paste", Paste),
                    MenuItem::separator(),
                    MenuItem::action("Select All", SelectAll),
                ],
            },
        ]);

        cx.open_window(options, |_, cx| {
            cx.new(|_| HelloWorld {
                line_numbers: vec![1, 2, 3]
            })
        })
        .expect("failed to open window");
    });
}
