use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, parse_macro_input};

#[proc_macro_derive(FormRenderable, attributes(form))]
pub fn derive_form_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let form_name = format_ident!("{}Form", name);

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields.named,
            _ => {
                return syn::Error::new_spanned(name, "Only named fields supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "Only structs supported")
                .to_compile_error()
                .into();
        }
    };

    let mut struct_fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut to_struct_fields = Vec::new();
    let mut selected_matches = Vec::new();
    let mut render_calls = Vec::new();
    let mut constraints = Vec::new();
    let mut height_exprs = Vec::new();
    let field_len = fields.len();

    for (idx, field) in fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let nested = is_nested_field(field);

        constraints.push(quote! { ratatui::layout::Constraint::Length(2) });

        if nested {
            let ty_str = ty.to_token_stream().to_string().replace(' ', "");
            let nested_form = format_ident!("{}Form", ty_str);

            struct_fields.push(quote! {
                pub #ident: #nested_form<'a>
            });
            field_inits.push(quote! {
                #ident: #nested_form::new()
            });
            to_struct_fields.push(quote! {
                #ident: self.#ident.to_struct()?
            });
            selected_matches.push(quote! {
                i if i == #idx => self.#ident.input(theinput),
            });
            render_calls.push(quote! {
                {
                    let chunk = chunks[#idx];
                    let cols = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            ratatui::layout::Constraint::Length(4),
                            ratatui::layout::Constraint::Min(0)
                        ])
                        .split(chunk);

                    self.#ident.render(f, cols[1], self.selected == #idx);
                }
            });

            height_exprs.push(quote! {
                #nested_form::form_height()
            });
        } else {
            struct_fields.push(quote! {
                pub #ident: ::reformy_core::Filtext<'a, #ty>
            });
            field_inits.push(quote! {
                #ident: ::reformy_core::Filtext::new()
            });
            to_struct_fields.push(quote! {
                #ident: self.#ident.value()?
            });
            selected_matches.push(quote! {
                i if i == #idx => self.#ident.input(theinput),
            });
            render_calls.push(quote! {
                {
                    let chunk = chunks[#idx];
                    let cols = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            ratatui::layout::Constraint::Length(12),
                            ratatui::layout::Constraint::Min(0)
                        ])
                        .split(chunk);

                    let label = if self.selected == #idx && infocus {
                        ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#ident)))
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                    } else {
                        ratatui::widgets::Paragraph::new(stringify!(#ident))
                    };

                    label.render(cols[0], f.buffer_mut());
                    self.#ident.input.render(cols[1], f.buffer_mut());
                }
            });
            height_exprs.push(quote! { 1 });
        }
    }

    let expanded = quote! {
        pub struct #form_name<'a> {
            #(#struct_fields),*,
            pub selected: usize,
        }

        impl<'a> #form_name<'a> {
            pub fn new() -> Self {
                Self {
                    #(#field_inits),*,
                    selected: 0,
                }
            }

            pub fn form_height() -> u16 {
                0 #( + #height_exprs )*
            }

            pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                let theinput = input.clone();
                let handled = match self.selected {
                    #(#selected_matches)*
                    _ => unreachable!(),
                };

                if handled {
                    return true;
                }

                match input.key {
                    tui_textarea::Key::Down => {
                        if self.selected < #field_len - 1 {
                            self.selected += 1;
                            true
                        } else {
                            false
                        }
                    }
                    tui_textarea::Key::Up => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }


            pub fn render(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect, infocus: bool) {
                use ratatui::layout::{Layout, Direction, Constraint};

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![#(Constraint::Length(#height_exprs)),*])
                    .split(area);

                #(#render_calls)*
            }

            pub fn to_struct(&self) -> Option<#name> {
                Some(#name {
                    #(#to_struct_fields),*
                })
            }
        }

        impl #name {
            pub fn form<'a>() -> #form_name<'a> {
                #form_name::new()
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_nested_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("form")
            && attr
                .parse_args::<syn::Ident>()
                .map_or(false, |i| i == "nested")
    })
}
