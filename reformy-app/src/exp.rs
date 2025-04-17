#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use std::{fmt::Display, str::FromStr};
use crossterm::event::{self, Event};
use ratatui::widgets::Widget;
use reformy::FormRenderable;
struct Email(String);
#[automatically_derived]
impl ::core::fmt::Debug for Email {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Email", &&self.0)
    }
}
#[automatically_derived]
impl ::core::default::Default for Email {
    #[inline]
    fn default() -> Email {
        Email(::core::default::Default::default())
    }
}
impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}", self.0))
    }
}
impl FromStr for Email {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("@") { Ok(Email(s.to_string())) } else { Err(()) }
    }
}
use ratatui::widgets::StatefulWidgetRef;
struct User {
    name: String,
    age: usize,
    #[form(nested)]
    role: Role,
    email: Email,
    #[form(nested)]
    address: Address,
}
#[automatically_derived]
impl ::core::fmt::Debug for User {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "User",
            "name",
            &self.name,
            "age",
            &self.age,
            "role",
            &self.role,
            "email",
            &self.email,
            "address",
            &&self.address,
        )
    }
}
#[automatically_derived]
impl ::core::default::Default for User {
    #[inline]
    fn default() -> User {
        User {
            name: ::core::default::Default::default(),
            age: ::core::default::Default::default(),
            role: ::core::default::Default::default(),
            email: ::core::default::Default::default(),
            address: ::core::default::Default::default(),
        }
    }
}
struct Address {
    #[form(nested)]
    whatever: Whatever,
    street: String,
    number: usize,
}
#[automatically_derived]
impl ::core::fmt::Debug for Address {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Address",
            "whatever",
            &self.whatever,
            "street",
            &self.street,
            "number",
            &&self.number,
        )
    }
}
#[automatically_derived]
impl ::core::default::Default for Address {
    #[inline]
    fn default() -> Address {
        Address {
            whatever: ::core::default::Default::default(),
            street: ::core::default::Default::default(),
            number: ::core::default::Default::default(),
        }
    }
}
struct Whatever {
    foo: String,
}
#[automatically_derived]
impl ::core::fmt::Debug for Whatever {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Whatever",
            "foo",
            &&self.foo,
        )
    }
}
#[automatically_derived]
impl ::core::default::Default for Whatever {
    #[inline]
    fn default() -> Whatever {
        Whatever {
            foo: ::core::default::Default::default(),
        }
    }
}
enum Role {
    Admin(String),
    Guest { name: String, cool: String, whatever: String },
    #[default]
    User,
}
#[automatically_derived]
impl ::core::fmt::Debug for Role {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Role::Admin(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Admin", &__self_0)
            }
            Role::Guest { name: __self_0, cool: __self_1, whatever: __self_2 } => {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "Guest",
                    "name",
                    __self_0,
                    "cool",
                    __self_1,
                    "whatever",
                    &__self_2,
                )
            }
            Role::User => ::core::fmt::Formatter::write_str(f, "User"),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for Role {
    #[inline]
    fn default() -> Role {
        Self::User
    }
}
pub struct RoleAdminForm {
    pub value: ::reformy_core::Filtext<String>,
}
impl RoleAdminForm {
    pub fn new() -> Self {
        Self {
            value: ::reformy_core::Filtext::new(),
        }
    }
    pub fn build(&self) -> Option<Role> {
        Some(Role::Admin(self.value.value()?))
    }
    pub fn input(&mut self, input: tui_textarea::Input) -> bool {
        self.value.input(input)
    }
    pub fn render(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut bool,
    ) {
        use ratatui::widgets::WidgetRef;
        use ratatui::prelude::Constraint;
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([Constraint::Length(1)]),
                ),
            )
            .split(area);
        let cols = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(chunks[0]);
        let label = if *state {
            ratatui::widgets::Paragraph::new(
                    ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(format_args!("> {0}", "value"));
                        res
                    }),
                )
                .style(
                    ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                )
        } else {
            ratatui::widgets::Paragraph::new("value")
        };
        label.render_ref(cols[0], buf);
        self.value.input.render(cols[1], buf);
    }
}
pub struct RoleGuestForm {
    pub selected: usize,
    pub name: ::reformy_core::Filtext<String>,
    pub cool: ::reformy_core::Filtext<String>,
    pub whatever: ::reformy_core::Filtext<String>,
}
impl RoleGuestForm {
    pub fn new() -> Self {
        Self {
            selected: 0,
            name: ::reformy_core::Filtext::new(),
            cool: ::reformy_core::Filtext::new(),
            whatever: ::reformy_core::Filtext::new(),
        }
    }
    pub fn build(&self) -> Option<Role> {
        Some(Role::Guest {
            name: self.name.value()?,
            cool: self.cool.value()?,
            whatever: self.whatever.value()?,
        })
    }
    pub fn input(&mut self, input: tui_textarea::Input) -> bool {
        let handled = match self.selected {
            i if i == 0usize => self.name.input(input.clone()),
            i if i == 1usize => self.cool.input(input.clone()),
            i if i == 2usize => self.whatever.input(input.clone()),
            _ => false,
        };
        if handled {
            return true;
        }
        match input.key {
            tui_textarea::Key::Down if self.selected < 3usize - 1 => {
                self.selected += 1;
                true
            }
            tui_textarea::Key::Up if self.selected > 0 => {
                self.selected -= 1;
                true
            }
            _ => false,
        }
    }
    pub fn render(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut bool,
    ) {
        use ratatui::widgets::WidgetRef;
        use ratatui::prelude::Constraint;
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]),
                ),
            )
            .split(area);
        {
            let cols = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(12),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(chunks[0usize]);
            let label = if self.selected == 0usize && *state {
                ratatui::widgets::Paragraph::new(
                        ::alloc::__export::must_use({
                            let res = ::alloc::fmt::format(
                                format_args!("> {0}", "name"),
                            );
                            res
                        }),
                    )
                    .style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Yellow),
                    )
            } else {
                ratatui::widgets::Paragraph::new("name")
            };
            label.render_ref(cols[0], buf);
            self.name.input.render(cols[1], buf);
        }
        {
            let cols = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(12),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(chunks[1usize]);
            let label = if self.selected == 1usize && *state {
                ratatui::widgets::Paragraph::new(
                        ::alloc::__export::must_use({
                            let res = ::alloc::fmt::format(
                                format_args!("> {0}", "cool"),
                            );
                            res
                        }),
                    )
                    .style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Yellow),
                    )
            } else {
                ratatui::widgets::Paragraph::new("cool")
            };
            label.render_ref(cols[0], buf);
            self.cool.input.render(cols[1], buf);
        }
        {
            let cols = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(12),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(chunks[2usize]);
            let label = if self.selected == 2usize && *state {
                ratatui::widgets::Paragraph::new(
                        ::alloc::__export::must_use({
                            let res = ::alloc::fmt::format(
                                format_args!("> {0}", "whatever"),
                            );
                            res
                        }),
                    )
                    .style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Yellow),
                    )
            } else {
                ratatui::widgets::Paragraph::new("whatever")
            };
            label.render_ref(cols[0], buf);
            self.whatever.input.render(cols[1], buf);
        }
    }
}
pub struct RoleForm {
    pub selected_variant: usize,
    pub admin: RoleAdminForm,
    pub guest: RoleGuestForm,
    pub user: (),
}
impl RoleForm {
    pub fn new() -> Self {
        Self {
            selected_variant: 0,
            admin: RoleAdminForm::new(),
            guest: RoleGuestForm::new(),
            user: (),
        }
    }
    pub fn form_height(&self) -> u16 {
        let index = self.selected_variant;
        (match index {
            0usize => 1,
            1usize => 3usize,
            2usize => 0,
            _ => 0,
        } + 2) as u16
    }
    pub fn input(&mut self, input: tui_textarea::Input) -> bool {
        let key = input.key.clone();
        (match self.selected_variant {
            0usize => self.admin.input(input.clone()),
            1usize => self.guest.input(input.clone()),
            2usize => false,
            _ => false,
        }
            || match key {
                tui_textarea::Key::Left if self.selected_variant > 0 => {
                    self.selected_variant -= 1;
                    true
                }
                tui_textarea::Key::Right if self.selected_variant + 1 < 3usize => {
                    self.selected_variant += 1;
                    true
                }
                _ => false,
            })
    }
    pub fn build(&self) -> Option<Role> {
        match self.selected_variant {
            0usize => self.admin.build(),
            1usize => self.guest.build(),
            2usize => Some(Role::User),
            _ => None,
        }
    }
    pub fn render(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: bool,
    ) {
        use ratatui::widgets::WidgetRef;
        use ratatui::prelude::Constraint;
        let label = match self.selected_variant {
            0usize => "Admin",
            1usize => "Guest",
            2usize => "User",
            _ => "???",
        };
        let title = if state {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(format_args!(">{0}: ", label));
                res
            })
        } else {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(format_args!("{0}: ", label));
                res
            })
        };
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([Constraint::Length(1), Constraint::Min(0)]),
                ),
            )
            .split(area);
        ratatui::widgets::Paragraph::new(
                ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(format_args!("[{0}]", label));
                    res
                }),
            )
            .render_ref(chunks[0], buf);
        let area = chunks[1];
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([Constraint::Length(2), Constraint::Min(0)]),
                ),
            )
            .split(area);
        let area = chunks[1];
        match self.selected_variant {
            0usize => self.admin.render(area, buf, &mut state.clone()),
            1usize => self.guest.render(area, buf, &mut state.clone()),
            2usize => {}
            _ => {}
        };
    }
}
impl ratatui::widgets::WidgetRef for RoleForm {
    fn render_ref(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        ratatui::widgets::StatefulWidgetRef::render_ref(self, area, buf, &mut true)
    }
}
impl ratatui::widgets::StatefulWidgetRef for RoleForm {
    type State = bool;
    fn render_ref(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        self.render(area, buf, *state);
    }
}
impl Role {
    pub fn form() -> RoleForm {
        RoleForm::new()
    }
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
    match foo.build() {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}:{2}] {3} = {4:#?}\n",
                        "reformy-app/src/main.rs",
                        92u32,
                        5u32,
                        "foo.build()",
                        &tmp,
                    ),
                );
            };
            tmp
        }
    };
}
