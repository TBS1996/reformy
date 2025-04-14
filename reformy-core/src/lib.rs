use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget, WidgetRef},
};
use tui_textarea::{Input, TextArea};

pub struct Filtext<T: Default + Display + FromStr> {
    pub input: TextArea<'static>,
    pub validate_input: bool,
    _phantom: PhantomData<T>,
}

impl<T: Default + Display + FromStr> Filtext<T> {
    pub fn new() -> Self {
        let input = T::default().to_string();
        Self {
            input: TextArea::from([input]),
            validate_input: false,
            _phantom: PhantomData,
        }
    }

    pub fn input(&mut self, input: Input) -> bool {
        if self.validate_input {
            let prev = self.input.lines().to_vec();
            let val = self.input.input(input);
            let new = self.value_string();
            if new.parse::<T>().is_err() && !prev.is_empty() {
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
