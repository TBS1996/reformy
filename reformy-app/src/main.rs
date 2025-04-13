use std::collections::HashMap;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use reformy_macro::FormRenderable;
use tui_textarea::{Input, TextArea};

#[derive(Debug, Default, FormRenderable)]
struct User {
    name: String,
    email: String,
}

fn main() {
    let mut foo = User::form();
    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|f| {
                f.render_widget(&foo, f.area());
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                event::KeyCode::Esc => break,
                key => {
                    foo.handle_key(key);
                }
            }
        }
    }

    ratatui::restore();
    dbg!(foo.object());
}
