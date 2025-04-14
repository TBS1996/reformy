use std::{fmt::Display, str::FromStr};

use crossterm::event::{self, Event};
use ratatui::widgets::{Paragraph, Widget};
use reformy_macro::FormRenderable;
use tui_textarea::TextArea;

#[derive(Debug, Default)]
struct Email(String);

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Email {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("@") {
            Ok(Email(s.to_string()))
        } else {
            Err(())
        }
    }
}

use ratatui::widgets::StatefulWidgetRef;

#[derive(Debug, Default, FormRenderable)]
struct User {
    name: String,
    age: usize,
    #[form(nested)]
    role: Role,
    email: Email,
    #[form(nested)]
    address: Address,
}

#[derive(Debug, Default, FormRenderable)]
struct Address {
    #[form(nested)]
    whatever: Whatever,
    street: String,
    number: usize,
}

#[derive(Debug, Default, FormRenderable)]
struct Whatever {
    foo: String,
}

#[derive(Debug, Default, FormRenderable)]
enum Role {
    Admin,
    Guest,
    #[default]
    User,
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
                    let input = tui_textarea::Input {
                        key: key.into(),
                        ..Default::default()
                    };
                    foo.input(input);
                }
            }
        }
    }

    ratatui::restore();
    dbg!(foo.to_struct());
}
