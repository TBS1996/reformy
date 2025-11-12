use std::{fmt::Display, str::FromStr};
use crossterm::event::{self, Event};
use reformy::reformy_cmd;

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

// Example function with reformy_cmd
#[reformy_cmd]
fn create_user(name: String, age: usize, email: Email) -> String {
    format!("Created user: {} (age: {}, email: {})", name, age, email)
}

// Another example function
#[reformy_cmd]
fn greet_person(first_name: String, last_name: String) -> String {
    format!("Hello, {} {}!", first_name, last_name)
}

fn main() {
    // Create a form for the create_user function
    let mut form = CreateUser::form();
    let mut terminal = ratatui::init();

    loop {
        terminal
            .draw(|f| {
                f.render_widget(&form, f.area());
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                event::KeyCode::Esc => break,
                event::KeyCode::Enter => {
                    // Try to build and execute
                    if let Some(args) = form.build() {
                        ratatui::restore();
                        let result = create_user(args.name, args.age, args.email);
                        println!("Result: {}", result);
                        println!("\nPress Enter to continue...");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        terminal = ratatui::init();
                    }
                }
                key_code => {
                    let input = tui_textarea::Input {
                        key: key_code.into(),
                        ctrl: key.modifiers.contains(event::KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(event::KeyModifiers::ALT),
                        shift: key.modifiers.contains(event::KeyModifiers::SHIFT),
                    };
                    form.input(input);
                }
            }
        }
    }

    ratatui::restore();
    dbg!(form.build());
}

