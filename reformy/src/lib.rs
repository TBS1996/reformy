pub use reformy_derive::{Form, form, menu};

pub use ratatui;
pub use crossterm;
pub use tui_textarea;

use std::{fmt::Display, marker::PhantomData, str::FromStr};
use ratatui::style::Style;
use tui_textarea::{Input, TextArea};

pub struct Filtext<T: Default + Display + FromStr> {
    pub input: TextArea<'static>,
    pub validate_input: bool,
    _phantom: PhantomData<T>,
}

impl<T: Default + Display + FromStr> Filtext<T> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_cursor_style(Style::new());
        input.set_cursor_line_style(Style::new());

        Self {
            input,
            validate_input: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_input = validate;
        self
    }

    pub fn input(&mut self, input: Input) -> bool {
        if self.validate_input {
            let prev = self.input.lines().to_vec();
            let val = self.input.input(input);
            let new = self.value_string();
            // Allow empty string (so user can delete everything) or valid parse
            if !new.is_empty() && new.parse::<T>().is_err() {
                self.input = TextArea::new(prev);
                false
            } else {
                val
            }
        } else {
            self.input.input(input)
        }
    }

    pub fn value(&self) -> Option<T> {
        T::from_str(&self.value_string()).ok()
    }

    pub fn value_string(&self) -> String {
        self.input.lines().concat()
    }
}
