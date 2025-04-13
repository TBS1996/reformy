
use crossterm::event::{self, Event};
use reformy_macro::FormRenderable;

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
