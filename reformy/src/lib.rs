use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Field, parse_macro_input};

#[proc_macro_derive(FormRenderable, attributes(form))]
pub fn derive_form_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let form_name = format_ident!("{}Form", name);

    match input.data {
        syn::Data::Enum(data_enum) => generate_enum_form(name, &form_name, &data_enum),
        syn::Data::Struct(data_struct) => {
            generate_struct_form(name, &form_name, &data_struct.fields)
        }
        _ => syn::Error::new_spanned(name, "Only structs and unit enums are supported")
            .to_compile_error()
            .into(),
    }
}

fn generate_enum_form(
    name: &syn::Ident,
    form_name: &syn::Ident,
    data_enum: &syn::DataEnum,
) -> TokenStream {
    let mut variant_fields = Vec::new();       // fields in the enum form
    let mut variant_inits = Vec::new();        // init code for those fields
    let mut build_matches = Vec::new();        // build() match arms
    let mut input_matches = Vec::new();        // input() match arms
    let mut render_matches = Vec::new();       // render() match arms
    let mut variant_display = Vec::new();      // display names
    let mut variant_titles = Vec::new();       // for label rendering
    let mut form_heights = Vec::new();       // for label rendering

    for (idx, variant) in data_enum.variants.iter().enumerate() {
        let v_ident = &variant.ident;
        let v_snake = format_ident!("{}", v_ident.to_string().to_lowercase());
        let variant_label = v_ident.to_string();

        match &variant.fields {
            syn::Fields::Unit => {
                variant_fields.push(quote! { pub #v_snake: () });
                variant_inits.push(quote! { #v_snake: () });


                form_heights.push(quote! {
                    #idx => 0,
                });

                build_matches.push(quote! {
                    #idx => Some(#name::#v_ident),
                });

                input_matches.push(quote! {
                    #idx => false,
                });

                render_matches.push(quote! {
                    #idx => {
                        //let text = format!("[{}]", #variant_label);
                        //ratatui::widgets::Paragraph::new(text).render_ref(area, buf);
                    }
                });
            }

            syn::Fields::Named(fields_named) => {
                let form_struct_name = format_ident!("{}{}Form", name, v_ident);

                let field_idents: Vec<_> = fields_named.named.iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let field_count = field_idents.len();
                form_heights.push(quote! {
                    #idx => #field_count,
                });

                let field_types: Vec<_> = fields_named.named.iter()
                    .map(|f| &f.ty)
                    .collect();

                let form_fields = field_idents.iter().zip(field_types.iter()).map(|(f, ty)| {
                    quote! {
                        pub #f: ::reformy_core::Filtext<#ty>
                    }
                });

                let field_inits = field_idents.iter().map(|f| {
                    quote! {
                        #f: ::reformy_core::Filtext::new()
                    }
                });

                let field_builds = field_idents.iter().map(|f| {
                    quote! {
                        #f: self.#f.value()?
                    }
                });

                let render_lines = field_idents.iter().enumerate().map(|(idx, f)| {
                    quote! {
                        {
                            let cols = ratatui::layout::Layout::default()
                                .direction(ratatui::layout::Direction::Horizontal)
                                .constraints([
                                    ratatui::layout::Constraint::Length(12),
                                    ratatui::layout::Constraint::Min(0),
                                ])
                                .split(chunks[#idx]);

                            let label = ratatui::widgets::Paragraph::new(stringify!(#f));
                            label.render_ref(cols[0], buf);
                            self.#f.input.render(cols[1], buf);
                        }
                    }
                });

                let form_struct = quote! {
                    pub struct #form_struct_name {
                        #(#form_fields,)*
                    }

                    impl #form_struct_name {
                        pub fn new() -> Self {
                            Self {
                                #(#field_inits,)*
                            }
                        }

                        pub fn build(&self) -> Option<#name> {
                            Some(#name::#v_ident {
                                #(#field_builds,)*
                            })
                        }

                        pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                            let mut handled = false;
                            #(handled |= self.#field_idents.input(input.clone());)*;
                            handled
                        }


                        pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
                            use ratatui::widgets::WidgetRef;
                            use ratatui::prelude::Constraint;

                            let chunks = ratatui::layout::Layout::default()
                                .direction(ratatui::layout::Direction::Vertical)
                                .constraints(vec![Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                                .split(area);


                            #(#render_lines)*
                        }
                    }
                };

                variant_titles.push(form_struct);

                variant_fields.push(quote! {
                    pub #v_snake: #form_struct_name
                });

                variant_inits.push(quote! {
                    #v_snake: #form_struct_name::new()
                });

                build_matches.push(quote! {
                    #idx => self.#v_snake.build(),
                });

                input_matches.push(quote! {
                    #idx => self.#v_snake.input(input.clone()),
                });

                render_matches.push(quote! {
                    #idx => self.#v_snake.render(area, buf),
                });
                }
         
            _ => {
                return syn::Error::new_spanned(&variant.fields, "Only unit or struct variants are supported")
                    .to_compile_error()
                    .into();
            }
        }

        variant_display.push(quote! {
            #idx => #variant_label,
        });
    }

    let num_variants = variant_display.len();

    quote! {
        #(#variant_titles)*

        pub struct #form_name {
            pub selected: usize,
            #(#variant_fields,)*
        }

        impl #form_name {
            pub fn new() -> Self {
                Self {
                    selected: 0,
                    #(#variant_inits,)*
                }
            }
            
            pub fn form_height(&self) -> u16 {
                let index = self.selected;
                (match index {
                    #(#form_heights)*
                    _ => 0,
                } + 1) as u16
            }

            pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                let key = input.key.clone();
                (match self.selected {
                    #(#input_matches)*
                    _ => false,
                } ||
                match key {
                    tui_textarea::Key::Left if self.selected > 0 => {
                        self.selected -= 1;
                        true
                    }
                    tui_textarea::Key::Right if self.selected + 1 < #num_variants => {
                        self.selected += 1;
                        true
                    }
                    _ => false,
                })
            }

            pub fn build(&self) -> Option<#name> {
                match self.selected {
                    #(#build_matches)*
                    _ => None,
                }
            }

            pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, infocus: bool) {
                use ratatui::widgets::WidgetRef;
                use ratatui::prelude::Constraint;

                let label = match self.selected {
                    #(#variant_display)*
                    _ => "???",
                };

                let title = if infocus {
                    format!(">[{}]", label)
                } else {
                    format!("[{}]", label)
                };

                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                ratatui::widgets::Paragraph::new(title).render_ref(chunks[0], buf);

                let area = chunks[1];

                match self.selected {
                    #(#render_matches)*
                    _ => {}
                };
            }
        }

        impl ratatui::widgets::WidgetRef for #form_name {
            fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
                ratatui::widgets::StatefulWidgetRef::render_ref(self, area, buf, &mut true)
            }
        }

        impl ratatui::widgets::StatefulWidgetRef for #form_name {
            type State = bool;
            fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
                self.render(area, buf, *state);
            }
        }

        impl #name {
            pub fn form() -> #form_name {
                #form_name::new()
            }
        }
    }.into()
}




fn generate_struct_form(
    name: &syn::Ident,
    form_name: &syn::Ident,
    fields: &syn::Fields,
) -> TokenStream {
    let named_fields = match fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => {
            return syn::Error::new_spanned(name, "Only named fields supported")
                .to_compile_error()
                .into();
        }
    };

    let mut struct_fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut to_struct_fields = Vec::new();
    let mut selected_matches = Vec::new();
    let mut render_calls = Vec::new();
    let mut height_exprs = Vec::new();

    for (idx, field) in named_fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if is_nested_field(field) {
            let nested_form =
                format_ident!("{}Form", ty.to_token_stream().to_string().replace(' ', ""));

            struct_fields.push(quote! { pub #ident: #nested_form });
            field_inits.push(quote! { #ident: #nested_form::new() });
            to_struct_fields.push(quote! { #ident: self.#ident.build()? });
            selected_matches.push(quote! { i if i == #idx => self.#ident.input(theinput.clone()), });
            render_calls.push(quote! {
                {
                    let chunk = chunks[#idx + 1];
                    let cols = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            ratatui::layout::Constraint::Length(4),
                            ratatui::layout::Constraint::Min(0)
                        ])
                        .split(chunk);

                    ratatui::widgets::StatefulWidgetRef::render_ref(
                        &self.#ident,
                        cols[1],
                        buf,
                        &mut (self.selected == #idx && *state),
                    );
                }
            });
            height_exprs.push(quote! { self.#ident.form_height() });
        } else {
            struct_fields.push(quote! { pub #ident: ::reformy_core::Filtext<#ty> });
            field_inits.push(quote! { #ident: ::reformy_core::Filtext::new() });
            to_struct_fields.push(quote! { #ident: self.#ident.value()? });
            selected_matches.push(quote! { i if i == #idx => self.#ident.input(theinput.clone()), });
            render_calls.push(quote! {
                {
                    let chunk = chunks[#idx + 1];
                    let cols = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            ratatui::layout::Constraint::Length(12),
                            ratatui::layout::Constraint::Min(0)
                        ])
                        .split(chunk);

                    let label = if self.selected == #idx && *state {
                        ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#ident)))
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                    } else {
                        ratatui::widgets::Paragraph::new(stringify!(#ident))
                    };

                    label.render_ref(cols[0], buf);
                    self.#ident.input.render(cols[1], buf);
                }
            });
            height_exprs.push(quote! { 1 });
        }
    }

    let field_count = named_fields.len();

    quote! {
        pub struct #form_name {
            #(#struct_fields,)*
            pub selected: usize,
        }

        impl #form_name {
            pub fn new() -> Self {
                Self {
                    #(#field_inits,)*
                    selected: 0,
                }
            }

            pub fn form_height(&self) -> u16 {
                0 #( + #height_exprs )* + 1
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
                    tui_textarea::Key::Down if self.selected < #field_count - 1 => {
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

            pub fn build(&self) -> Option<#name> {
                Some(#name {
                    #(#to_struct_fields,)*
                })
            }
        }

        impl ratatui::widgets::WidgetRef for #form_name {
            fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
                ratatui::widgets::StatefulWidgetRef::render_ref(self, area, buf, &mut true)
            }
        }

        impl ratatui::widgets::StatefulWidgetRef for #form_name {
            type State = bool;

            fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
                use ratatui::layout::{Layout, Direction, Constraint};
                use ratatui::widgets::WidgetRef;

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Length(1), #(Constraint::Length(#height_exprs)),*])
                    .split(area);

                let title = ratatui::widgets::Paragraph::new(stringify!(#name).to_string() + ":")
    .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
                title.render_ref(chunks[0], buf);

                #(#render_calls)*
            }
        }

        impl #name {
            pub fn form() -> #form_name {
                #form_name::new()
            }
        }
    }
    .into()
}

fn is_nested_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("form")
            && attr
                .parse_args::<syn::Ident>()
                .map_or(false, |i| i == "nested")
    })
}
