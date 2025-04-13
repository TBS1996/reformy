pub trait FormRenderable {
    fn field_names(&self) -> Vec<&'static str>;
    fn get_field(&self, field: &str) -> &str;
    fn set_field(&mut self, field: &str, value: String);
}

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget, WidgetRef},
};
use tui_textarea::{Input, TextArea};

pub struct Former<'a, T: FormRenderable> {
    object: T,
    selected: usize,
    texts: Vec<(String, TextArea<'a>)>,
}

impl<'a, T: FormRenderable> Widget for &Former<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_ref(area, buf);
    }
}

impl<'a, T: FormRenderable> WidgetRef for &Former<'a, T> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let constraints: Vec<Constraint> =
            self.texts.iter().map(|_| Constraint::Length(1)).collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let chunks: Vec<Rect> = chunks.into_iter().cloned().collect();

        for (i, (field, area_widget)) in self.texts.iter().enumerate() {
            let row = chunks[i];
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(12), Constraint::Min(0)])
                .split(row);

            let label = if i == self.selected {
                Paragraph::new(Line::from(format!("> {field}")))
                    .style(Style::default().fg(Color::Yellow))
            } else {
                Paragraph::new(Line::from(field.clone()))
            };
            label.render_ref(cols[0], buf);

            area_widget.render(cols[1], buf);
        }
    }
}

impl<'a, T: FormRenderable> Former<'a, T> {
    pub fn new(object: T) -> Self {
        let texts = object
            .field_names()
            .into_iter()
            .map(|field| {
                let mut area = TextArea::default();
                area.insert_str(object.get_field(&field));
                (field.to_string(), area)
            })
            .collect();

        Self {
            object,
            selected: 0,
            texts,
        }
    }

    fn update_object(&mut self) {
        let fieldvals = self
            .texts
            .iter()
            .map(|(field, val)| (field, val.clone().into_lines().concat()));
        for (field, val) in fieldvals {
            self.object.set_field(field, val);
        }
    }

    pub fn object(&self) -> &T {
        &self.object
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                let len = self.texts.len();
                if self.selected < len - 1 {
                    self.selected += 1;
                }
            }
            key => {
                let key: tui_textarea::Key = key.into();
                let input = Input {
                    key,
                    ctrl: Default::default(),
                    alt: Default::default(),
                    shift: Default::default(),
                };
                if self.texts[self.selected].1.input(input) {
                    self.update_object();
                }
            }
        }
    }
}
