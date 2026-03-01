mod actions;
mod menu;
mod app_state;
mod ui;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::ScrollbarShow;
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{init, Root};

use crate::actions::*;
use crate::app_state::ScrollDemo;

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
            KeyBinding::new("cmd-f", FindAction, None),
            KeyBinding::new("ctrl-f", FindAction, None),
            KeyBinding::new("cmd-shift-f", FindInFilesAction, None),
            KeyBinding::new("ctrl-shift-f", FindInFilesAction, None),
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
        cx.open_window(WindowOptions { 
            window_bounds: Some(WindowBounds::Windowed(bounds)), 
            ..Default::default() 
        }, |window, cx| {
            let view = cx.new(|cx| ScrollDemo::new(cx));
            cx.new(|cx| Root::new(view, window, cx))
        }).expect("failed to open window");
    });
}