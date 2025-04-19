use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Field, FieldsNamed, Variant, parse_macro_input, parse_str, parse2};

#[proc_macro_derive(FormRenderable, attributes(form))]
pub fn derive_form_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let obj = match input.data {
        syn::Data::Enum(data_enum) => generate_enum_form(&name, data_enum),
        syn::Data::Struct(data_struct) => generate_struct_form(name, data_struct.fields),
        _ => {
            return syn::Error::new_spanned(name, "Only structs and unit enums are supported")
                .to_compile_error()
                .into();
        }
    };

    obj.generate().into()
}

fn extract_unit(
    v_ident: &syn::Ident,
    idx: usize,
) -> VariantInfo {
    let init = quote! { #v_ident: () };

    let input = quote! {
        #idx => false,
    };

    let render = quote! {
        #idx => {}
    };

    VariantInfo {
        v_ident: v_ident.clone(),
        v_ty: parse_str("()").unwrap(),
        height: 0,
        input,
        init,
        render,
        titles: None,
    }
}

fn extract_named(
    fields_named: FieldsNamed,
    name: &syn::Ident,
    v_ident: &syn::Ident,
    idx: usize,
) -> VariantInfo {
    let mut fields: Vec<Field> = vec![];

    for field in fields_named.clone().named {
        fields.push(field);
    }

    let mystruct = MyStruct::new(name.clone(), Some(v_ident.clone()), fields);

    let form_struct_name = mystruct.form_name();

    let field_idents: Vec<_> = fields_named
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let field_count = field_idents.len();

    let init = quote! {
        #v_ident: #form_struct_name::new()
    };

    let input = quote! {
        #idx => self.#v_ident.input(input.clone()),
    };

    let render = quote! {
        #idx => self.#v_ident.render(area, buf, state.clone()),
    };


    VariantInfo {
        v_ident: v_ident.clone(),
        v_ty: form_struct_name,
        height: field_count,
        input,
        init,
        render,
        titles: Some(mystruct),
    }
}

fn extract_variant(name: &syn::Ident, variant: Variant, idx: usize) -> VariantInfo {
    let v_ident = &variant.ident;
    match variant.fields {
        syn::Fields::Unit => extract_unit(v_ident, idx),
        syn::Fields::Named(fields_named) => {
            extract_named(fields_named, name, v_ident, idx)
        }

        _ => {
            panic!()
            /*
            return syn::Error::new_spanned(&variant.fields, "Only unit or struct variants are supported")
                .to_compile_error()
                .into();
                */
        }
    }
}

fn generate_enum_form(name: &syn::Ident, data_enum: syn::DataEnum) -> MyObject {
    let mut fields: Vec<VariantInfo> = vec![];

    for (idx, variant) in data_enum.variants.into_iter().enumerate() {
        fields.push(extract_variant(name, variant, idx));
    }

    let myenum = MyEnum {
        name: name.clone(),
        variants: fields,
    };
    MyObject::Enum(myenum)
}

/// Represents all the info needed to create a Form object
enum MyObject {
    Enum(MyEnum),
    Struct(MyStruct),
}

impl MyObject {
    fn form_name(&self) -> syn::Type {
        match self {
            MyObject::Enum(obj) => obj.form_name(),
            MyObject::Struct(obj) => obj.form_name(),
        }
    }

    fn name(&self) -> syn::Ident {
        match self {
            MyObject::Enum(obj) => obj.name.clone(),
            MyObject::Struct(obj) => obj.name.clone(),
        }
    }

    fn generate(&self) -> proc_macro2::TokenStream {
        let stream = match self {
            MyObject::Enum(ob) => ob.generate(),
            MyObject::Struct(ob) => ob.generate(),
        };

        let name = self.name();
        let form_name = self.form_name();

        let widget: proc_macro2::TokenStream = quote! {
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
        };

        quote! { #stream
        #widget}
    }
}

struct MyEnum {
    name: syn::Ident,
    variants: Vec<VariantInfo>,
}

impl MyEnum {
    fn form_name(&self) -> syn::Type {
        let ident = format_ident!("{}Form", &self.name);
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: ident.into(),
        })
    }

    fn generate(&self) -> proc_macro2::TokenStream {
        let form_name = self.form_name();

        let variant_fields: Vec<_> = self
            .variants
            .iter()
            .map(|info| {
                let ident = &info.v_ident;
                let ty = &info.v_ty;

                quote! { pub #ident: #ty  }
            })
            .collect();
        let form_heights: Vec<_> = self
            .variants
            .iter().enumerate()
            .map(|(idx, info)| {
                let count = info.height;
               
                quote! {
                    #idx => #count,
                }
                
            })
            .collect();
        let input_matches: Vec<_> = self
            .variants
            .iter()
            .map(|info| info.input.clone())
            .collect();
        let build_matches: Vec<_> = self
            .variants
            .iter().enumerate()
            .map(|(idx, info)| {
                let ident = &info.v_ident;
                if info.titles.is_some() {
                    quote! {
                        #idx => self.#ident.build(),
                    }
                } else {
                    let name = &self.name;
                    quote! {
                        #idx => Some(#name::#ident),
                    }
                }
            })
            .collect();
        let variant_inits: Vec<_> = self.variants.iter().map(|info| info.init.clone()).collect();
        let render_matches: Vec<_> = self
            .variants
            .iter()
            .map(|info| info.render.clone())
            .collect();
        let variant_titles: Vec<_> = self
            .variants
            .iter()
            .map(|info| {
                info.titles
                    .as_ref()
                    .map(|mys| mys.generate())
                    .unwrap_or_default()
            })
            .collect();
        let variant_display: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let label = info.v_ident.to_string();
                quote!(#idx => #label,)
            })
            .collect();

        let num_variants = variant_display.len();
        let name = &self.name;

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
                } + 2) as u16
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
                    format!(">{}: ", label)
                } else {
                    format!("{}: ", label)
                };

                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                ratatui::widgets::Paragraph::new(format!("[{}]", label)).render_ref(chunks[0], buf);

                let area = chunks[1];

                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints(vec![Constraint::Length(2), Constraint::Min(0)])
                    .split(area);

                let area = chunks[1];

                match self.selected_variant {
                    #(#render_matches)*
                    _ => {}
                };
            }
        }

    }.into()
    }
}

/// A single variant in an enum
struct VariantInfo {
    v_ident: syn::Ident,
    v_ty: syn::Type,
    height: usize,
    input: proc_macro2::TokenStream,
    init: proc_macro2::TokenStream,
    render: proc_macro2::TokenStream,
    /// The fields if it's a data enum, none if it's unit
    titles: Option<MyStruct>,
}

/// A single field in a struct-like object.
struct StructField {
    field: syn::Ident,
    field_ty: syn::Type,
    height: proc_macro2::TokenStream,
    build: proc_macro2::TokenStream,
    render: proc_macro2::TokenStream,
}

struct MyStruct {
    name: syn::Ident,
    variant: Option<syn::Ident>,
    fields: Vec<StructField>,
}

impl MyStruct {
    fn new(name: syn::Ident, variant: Option<syn::Ident>, fields: Vec<Field>) -> Self {
        let mut xfields: Vec<StructField> = vec![];

        for (idx, field) in fields.iter().enumerate() {
            xfields.push(extract_field(idx, field));
        }

        Self {
            name,
            variant,
            fields: xfields,
        }
    }

    fn form_name(&self) -> syn::Type {
        let ident = match &self.variant {
            Some(var) => format_ident!("{}{}Form", self.name, var),
            None => format_ident!("{}Form", self.name),
        };
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: ident.into(),
        })
    }

    fn generate(&self) -> proc_macro2::TokenStream {
        if self.fields.is_empty() {
            return quote! {}.into();
        }

        let struct_fields: Vec<_> = self
            .fields
            .iter()
            .map(|i| {
                let name = i.field.clone();
                let ty = i.field_ty.clone();

                quote! { pub #name: #ty }
            })
            .collect();
        let height_exprs: Vec<_> = self.fields.iter().map(|i| i.height.clone()).collect();
        let field_inits: Vec<_> = self
            .fields
            .iter()
            .map(|i| {
                let field = i.field.clone();
                let ty = i.field_ty.clone();
                quote! { #field: #ty::new() }
            })
            .collect();
        let to_struct_fields: Vec<_> = self.fields.iter().map(|i| i.build.clone()).collect();
        let selected_matches: Vec<_> = self
            .fields
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                let ident = i.field.clone();

                quote! { i if i == #idx => self.#ident.input(theinput.clone()), }
            })
            .collect();
        let render_calls: Vec<_> = self.fields.iter().map(|i| i.render.clone()).collect();
        let field_count = struct_fields.len();
        let name = &self.name;
        let form_name = self.form_name();

        let buildent = if let Some(variant) = &self.variant {
            quote! { #name::#variant }
        } else {
            quote! { #name }
        };

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

                fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: bool) {
                    use ratatui::layout::{Layout, Direction, Constraint};
                    use ratatui::widgets::WidgetRef;

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(vec![#(Constraint::Length(#height_exprs)),*])
                        .split(area);

                    let title = ratatui::widgets::Paragraph::new(stringify!(self.name).to_string() + ":")
        .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));

                    #(#render_calls)*

                }

                pub fn build(&self) -> Option<#name> {
                    Some(#buildent {
                        #(#to_struct_fields,)*
                    })
                }
            }
        }
    }
}

fn extract_field(idx: usize, field: &Field) -> StructField {
    let ident = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    if is_nested_field(field) {
        let ty: syn::Type = parse_str(&format!(
            "{}Form",
            ty.to_token_stream().to_string().replace(' ', "")
        ))
        .unwrap();

        let to_fields = quote! { #ident: self.#ident.build()? };
        let height = quote! { self.#ident.form_height() };

        let render = quote! {
            {
                let chunk = chunks[#idx];
                let cols = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(1),
                        ratatui::layout::Constraint::Min(0)
                    ])
                    .split(chunk);

                let label = if self.selected == #idx && state {
                    ratatui::widgets::Paragraph::new(format!("> {}:", stringify!(#ident)))
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                } else {
                    ratatui::widgets::Paragraph::new(format!("{}:", stringify!(#ident)))
                };

                label.render_ref(cols[0], buf);

                let cols = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints([
                        ratatui::layout::Constraint::Length(4),
                        ratatui::layout::Constraint::Min(0)
                    ])
                    .split(cols[1]);

                ratatui::widgets::StatefulWidgetRef::render_ref(
                    &self.#ident,
                    cols[1],
                    buf,
                    &mut (self.selected == #idx && state),
                );
            }
        };

        StructField {
            field: ident.clone(),
            field_ty: ty,
            height,
            build: to_fields,
            render,
        }
    } else {
        let to_fields = quote! { #ident: self.#ident.value()? };
        let render = quote! {
            {
                let chunk = chunks[#idx];
                let cols = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints([
                        ratatui::layout::Constraint::Length(12),
                        ratatui::layout::Constraint::Min(0)
                    ])
                    .split(chunk);

                let label = if self.selected == #idx && state {
                    ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#ident)))
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                } else {
                    ratatui::widgets::Paragraph::new(stringify!(#ident))
                };

                label.render_ref(cols[0], buf);
                self.#ident.input.render(cols[1], buf);
            }
        };
        let height = quote! { 1 };
        StructField {
            field: ident.clone(),
            field_ty: parse2(quote! {::reformy_core::Filtext::<#ty>}).unwrap(),
            height,
            build: to_fields,
            render,
        }
    }
}

fn generate_struct_form(name: syn::Ident, fields: syn::Fields) -> MyObject {
    let named_fields = match fields {
        syn::Fields::Named(fields) => fields.named,
        _ => {
            panic!("only named fields")
        }
    };

    let mystruct = MyStruct::new(name.clone(), None, named_fields.into_iter().collect());

    MyObject::Struct(mystruct)
}

fn is_nested_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("form")
            && attr
                .parse_args::<syn::Ident>()
                .map_or(false, |i| i == "nested")
    })
}
