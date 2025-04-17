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

struct VariantInfo {
    field: proc_macro2::TokenStream,
    heights: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
    build: proc_macro2::TokenStream,
    init: proc_macro2::TokenStream,
}

fn generate_enum_form(
    name: &syn::Ident,
    form_name: &syn::Ident,
    data_enum: &syn::DataEnum,
) -> TokenStream {
    let mut fields: Vec<VariantInfo> = vec![];

    let mut render_matches = Vec::new();       // render() match arms
    let mut variant_display = Vec::new();      // display names
    let mut variant_titles = Vec::new();       // for label rendering

    for (idx, variant) in data_enum.variants.iter().enumerate() {
        let v_ident = &variant.ident;
        let v_snake = format_ident!("{}", v_ident.to_string().to_lowercase());
        let variant_label = v_ident.to_string();

        match &variant.fields {
            syn::Fields::Unit => {
                let variant_field = quote! { pub #v_snake: () };
                let heights = quote! {
                    #idx => 0,
                };
              

                let init = quote! { #v_snake: () };

                let build = quote! {
                    #idx => Some(#name::#v_ident),
                };

                let input = quote! {
                    #idx => false,
                };
                
                fields.push(VariantInfo {field: variant_field, heights, input, build, init});

                render_matches.push(quote! {
                    #idx => {}
                });
            }
            syn::Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                let form_field_type = &fields_unnamed.unnamed[0].ty;
                let form_field_name = format_ident!("value");
                let form_struct_name = format_ident!("{}{}Form", name, v_ident);
                
                let field = quote! {
                    pub #v_snake: #form_struct_name
                };


                let heights = quote! {
                    #idx => 1,
                };

                

                variant_titles.push(quote! {
                    pub struct #form_struct_name {
                        pub #form_field_name: ::reformy_core::Filtext<#form_field_type>
                    }

                    impl #form_struct_name {
                        pub fn new() -> Self {
                            Self {
                                #form_field_name: ::reformy_core::Filtext::new(),
                            }
                        }

                        pub fn build(&self) -> Option<#name> {
                            Some(#name::#v_ident(self.#form_field_name.value()?))
                        }

                        pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                            self.#form_field_name.input(input)
                        }

                        pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut bool) {
                            use ratatui::widgets::WidgetRef;
                            use ratatui::prelude::Constraint;

                            let chunks = ratatui::layout::Layout::default()
                                .direction(ratatui::layout::Direction::Vertical)
                                .constraints(vec![Constraint::Length(1)])
                                .split(area);

                            let cols = ratatui::layout::Layout::default()
                                .direction(ratatui::layout::Direction::Horizontal)
                                .constraints([
                                    ratatui::layout::Constraint::Length(12),
                                    ratatui::layout::Constraint::Min(0),
                                ])
                                .split(chunks[0]);

                            let label = if *state {
                                ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#v_snake)))
                                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                            } else {
                                ratatui::widgets::Paragraph::new(stringify!(#v_snake))
                            };

                            label.render_ref(cols[0], buf);
                            self.#form_field_name.input.render(cols[1], buf);
                        }
                    }
                });


                let init = quote! {
                    #v_snake: #form_struct_name::new()
                };

                let build = quote! {
                    #idx => self.#v_snake.build(),
                };

                let input = quote! {
                    #idx => self.#v_snake.input(input.clone()),
                };

                fields.push(VariantInfo {field, heights, input, build, init});

                render_matches.push(quote! {
                    #idx => self.#v_snake.render(area, buf, &mut state.clone()),
                });
            }
            syn::Fields::Named(fields_named) => {
                let form_struct_name = format_ident!("{}{}Form", name, v_ident);
                
                let field = quote! {
                    pub #v_snake: #form_struct_name
                };

                let field_idents: Vec<_> = fields_named.named.iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                
                let field_indices: Vec<_> = (0..field_idents.len()).collect();
                let field_input_match = field_indices.iter().zip(field_idents.iter()).map(|(i, f)| {
                    quote! {
                        i if i == #i => self.#f.input(input.clone()),
                    }
                });

                let field_count = field_idents.len();
                let heights = quote! {
                    #idx => #field_count,
                };


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

                      //      let label = ratatui::widgets::Paragraph::new(stringify!(#f));

                            
                            let label = if self.selected == #idx && *state {
                                ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#f)))
                                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                            } else {
                                ratatui::widgets::Paragraph::new(stringify!(#f))
                            };
                            
                            label.render_ref(cols[0], buf);
                            self.#f.input.render(cols[1], buf);
                        }
                    }
                });

                let form_struct = quote! {
                    pub struct #form_struct_name {
                        pub selected: usize,
                        #(#form_fields,)*
                    }

                    impl #form_struct_name {
                        pub fn new() -> Self {
                            Self {
                                selected: 0,
                                #(#field_inits,)*
                            }
                        }

                        pub fn build(&self) -> Option<#name> {
                            Some(#name::#v_ident {
                                #(#field_builds,)*
                            })
                        }
                       

                        pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                            let handled = match self.selected {
                                #(#field_input_match)*
                                _ => false,
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


                        pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut bool) {
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


                let init =quote! {
                    #v_snake: #form_struct_name::new()
                };

                let build = quote! {
                    #idx => self.#v_snake.build(),
                };

                let input = quote! {
                    #idx => self.#v_snake.input(input.clone()),
                };

                fields.push(VariantInfo {field, heights, input, build, init});

                render_matches.push(quote! {
                    #idx => self.#v_snake.render(area, buf, &mut state.clone()),
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

    let variant_fields: Vec<_> = fields.iter().map(|info|info.field.clone()).collect();
    let form_heights: Vec<_> = fields.iter().map(|info|info.heights.clone()).collect();
    let input_matches: Vec<_> = fields.iter().map(|info|info.input.clone()).collect();
    let build_matches: Vec<_> = fields.iter().map(|info|info.build.clone()).collect();
    let variant_inits: Vec<_> = fields.iter().map(|info|info.init.clone()).collect();

    let num_variants = variant_display.len();

    quote! {
        #(#variant_titles)*

        pub struct #form_name {
            pub selected_variant: usize,
            #(#variant_fields,)*
        }

        impl #form_name {
            pub fn new() -> Self {
                Self {
                    selected_variant: 0,
                    #(#variant_inits,)*
                }
            }
            
            pub fn form_height(&self) -> u16 {
                let index = self.selected_variant;
                (match index {
                    #(#form_heights)*
                    _ => 0,
                } + 1) as u16
            }

            pub fn input(&mut self, input: tui_textarea::Input) -> bool {
                let key = input.key.clone();
                (match self.selected_variant {
                    #(#input_matches)*
                    _ => false,
                } ||
                match key {
                    tui_textarea::Key::Left if self.selected_variant > 0 => {
                        self.selected_variant -= 1;
                        true
                    }
                    tui_textarea::Key::Right if self.selected_variant + 1 < #num_variants => {
                        self.selected_variant += 1;
                        true
                    }
                    _ => false,
                })
            }

            pub fn build(&self) -> Option<#name> {
                match self.selected_variant {
                    #(#build_matches)*
                    _ => None,
                }
            }

            pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: bool) {
                use ratatui::widgets::WidgetRef;
                use ratatui::prelude::Constraint;

                let label = match self.selected_variant {
                    #(#variant_display)*
                    _ => "???",
                };

                let title = if state {
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

                match self.selected_variant {
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
