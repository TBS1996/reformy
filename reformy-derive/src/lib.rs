use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Field, FieldsNamed, Variant, parse_macro_input, parse_str, parse2};
use syn::{ItemFn, FnArg, Pat};

#[proc_macro_derive(Form, attributes(form))]
pub fn derive_form(input: TokenStream) -> TokenStream {
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

fn extract_named(
    fields_named: FieldsNamed,
    name: &syn::Ident,
    v_ident: &syn::Ident,
) -> VariantInfo {
    let mut fields: Vec<Field> = vec![];

    for field in fields_named.clone().named {
        fields.push(field);
    }

    let mystruct = MyStruct::new(name.clone(), Some(v_ident.clone()), fields);

    VariantInfo {
        v_ident: v_ident.clone(),
        titles: Some(mystruct),
    }
}

fn extract_variant(name: &syn::Ident, variant: Variant) -> VariantInfo {
    let v_ident = &variant.ident;
    match variant.fields {
        syn::Fields::Unit => VariantInfo {
            v_ident: v_ident.clone(),
            titles: None,
        },
        syn::Fields::Named(fields_named) => extract_named(fields_named, name, v_ident),

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

    for variant in data_enum.variants.into_iter() {
        fields.push(extract_variant(name, variant));
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
            impl ::reformy::ratatui::widgets::WidgetRef for #form_name {
                fn render_ref(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer) {
                    ::reformy::ratatui::widgets::StatefulWidgetRef::render_ref(self, area, buf, &mut true)
                }
            }

            impl ::reformy::ratatui::widgets::StatefulWidgetRef for #form_name {
                type State = bool;
                fn render_ref(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer, state: &mut Self::State) {
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
                let ty = &info.form_name();

                quote! { pub #ident: #ty  }
            })
            .collect();
        let form_heights: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let count = info
                    .titles
                    .as_ref()
                    .map(|x| x.height(true))
                    .unwrap_or(quote! {0});

                quote! {
                    #idx => #count,
                }
            })
            .collect();

        let input_matches: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let ident = &info.v_ident;

                if info.titles.is_some() {
                    quote! {
                        #idx => self.#ident.input(input.clone()),
                    }
                } else {
                    quote! {
                        #idx => false,
                    }
                }
            })
            .collect();
        let build_matches: Vec<_> = self
            .variants
            .iter()
            .enumerate()
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

        let variant_inits: Vec<_> = self
            .variants
            .iter()
            .map(|info| {
                let ident = &info.v_ident;
                match &info.titles {
                    Some(s) => {
                        let form = s.form_name();
                        quote! {
                            #ident: #form::new()
                        }
                    }
                    None => {
                        quote! {
                            #ident: ()
                        }
                    }
                }
            })
            .collect();
        let render_matches: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let ident = &info.v_ident;

                if info.titles.is_some() {
                    quote! {
                        #idx => self.#ident.render(area, buf, state.clone()),
                    }
                } else {
                    quote! {
                        #idx => {},
                    }
                }
            })
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

            pub fn input(&mut self, input: ::reformy::tui_textarea::Input) -> bool {
                let key = input.key.clone();
                (match self.selected_variant {
                    #(#input_matches)*
                    _ => false,
                } ||
                match key {
                    ::reformy::tui_textarea::Key::Left if self.selected_variant > 0 => {
                        self.selected_variant -= 1;
                        true
                    }
                    ::reformy::tui_textarea::Key::Right if self.selected_variant + 1 < #num_variants => {
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

            pub fn render(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer, state: bool) {
                use ::reformy::ratatui::widgets::WidgetRef;
                use ::reformy::ratatui::prelude::Constraint;

                let label = match self.selected_variant {
                    #(#variant_display)*
                    _ => "???",
                };

                let title = if state {
                    format!(">{}: ", label)
                } else {
                    format!("{}: ", label)
                };

                let chunks = ::reformy::ratatui::layout::Layout::default()
                    .direction(::reformy::ratatui::layout::Direction::Vertical)
                    .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                ::reformy::ratatui::widgets::Paragraph::new(format!("[{}]", label)).render_ref(chunks[0], buf);

                let area = chunks[1];

                let chunks = ::reformy::ratatui::layout::Layout::default()
                    .direction(::reformy::ratatui::layout::Direction::Horizontal)
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
    /// The fields if it's a data enum, none if it's unit
    titles: Option<MyStruct>,
}

impl VariantInfo {
    fn form_name(&self) -> syn::Type {
        match &self.titles {
            Some(s) => s.form_name(),
            None => parse_str("()").unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
struct FieldType {
    ty: syn::Type,
    is_leaf: bool,
}

/// A single field in a struct-like object.
struct StructField {
    field: syn::Ident,
    field_ty: FieldType,
    build: proc_macro2::TokenStream,
    render: proc_macro2::TokenStream,
    needs_validation: bool,
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

    fn height_exprs(&self, is_enum: bool) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                if f.field_ty.is_leaf {
                    quote! { 1 }
                } else {
                    //let height = quote! { self.#ident.form_height() };
                    let ident = f.field.clone();
                    match &self.variant {
                        Some(var) if is_enum => quote! {self.#var.#ident.form_height()},
                        _ => quote! {self.#ident.form_height()},
                    }
                }
            })
            .collect()
    }

    fn height(&self, is_enum: bool) -> proc_macro2::TokenStream {
        let heights = self.height_exprs(is_enum);
        quote! {
            0 #( + #heights )*
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
                let ty = i.field_ty.ty.clone();

                quote! { pub #name: #ty }
            })
            .collect();
        let height_exprs: Vec<_> = self.height_exprs(false);
        let field_inits: Vec<_> = self
            .fields
            .iter()
            .map(|i| {
                let field = i.field.clone();
                let ty = i.field_ty.ty.clone();
                if i.needs_validation {
                    quote! { #field: #ty::new().with_validation(true) }
                } else {
                    quote! { #field: #ty::new() }
                }
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

                pub fn input(&mut self, input: ::reformy::tui_textarea::Input) -> bool {
                    let theinput = input.clone();
                    let handled = match self.selected {
                        #(#selected_matches)*
                        _ => unreachable!(),
                    };

                    if handled {
                        return true;
                    }

                    match input.key {
                        ::reformy::tui_textarea::Key::Down if self.selected < #field_count - 1 => {
                            self.selected += 1;
                            true
                        }
                        ::reformy::tui_textarea::Key::Up if self.selected > 0 => {
                            self.selected -= 1;
                            true
                        }
                        _ => false,
                    }
                }

                fn render(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer, state: bool) {
                    use ::reformy::ratatui::layout::{Layout, Direction, Constraint};
                    use ::reformy::ratatui::widgets::WidgetRef;

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(vec![#(Constraint::Length(#height_exprs)),*])
                        .split(area);

                    let title = ::reformy::ratatui::widgets::Paragraph::new(stringify!(self.name).to_string() + ":")
        .style(::reformy::ratatui::style::Style::default().add_modifier(::reformy::ratatui::style::Modifier::BOLD));

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

        let render = quote! {
            {
                let chunk = chunks[#idx];
                let cols = ::reformy::ratatui::layout::Layout::default()
                    .direction(::reformy::ratatui::layout::Direction::Vertical)
                    .constraints([
                        ::reformy::ratatui::layout::Constraint::Length(1),
                        ::reformy::ratatui::layout::Constraint::Min(0)
                    ])
                    .split(chunk);

                let label = if self.selected == #idx && state {
                    ::reformy::ratatui::widgets::Paragraph::new(format!("> {}:", stringify!(#ident)))
                        .style(::reformy::ratatui::style::Style::default().fg(::reformy::ratatui::style::Color::Yellow))
                } else {
                    ::reformy::ratatui::widgets::Paragraph::new(format!("{}:", stringify!(#ident)))
                };

                label.render_ref(cols[0], buf);

                let cols = ::reformy::ratatui::layout::Layout::default()
                    .direction(::reformy::ratatui::layout::Direction::Horizontal)
                    .constraints([
                        ::reformy::ratatui::layout::Constraint::Length(4),
                        ::reformy::ratatui::layout::Constraint::Min(0)
                    ])
                    .split(cols[1]);

                ::reformy::ratatui::widgets::StatefulWidgetRef::render_ref(
                    &self.#ident,
                    cols[1],
                    buf,
                    &mut (self.selected == #idx && state),
                );
            }
        };

        StructField {
            field: ident.clone(),
            field_ty: FieldType { ty, is_leaf: false },
            build: to_fields,
            render,
            needs_validation: false,
        }
    } else {
        let to_fields = quote! { #ident: self.#ident.value()? };
        let render = quote! {
            {
                let chunk = chunks[#idx];
                let cols = ::reformy::ratatui::layout::Layout::default()
                    .direction(::reformy::ratatui::layout::Direction::Horizontal)
                    .constraints([
                        ::reformy::ratatui::layout::Constraint::Length(12),
                        ::reformy::ratatui::layout::Constraint::Min(0)
                    ])
                    .split(chunk);

                let label = if self.selected == #idx && state {
                    ::reformy::ratatui::widgets::Paragraph::new(format!("> {}", stringify!(#ident)))
                        .style(::reformy::ratatui::style::Style::default().fg(::reformy::ratatui::style::Color::Yellow))
                } else {
                    ::reformy::ratatui::widgets::Paragraph::new(stringify!(#ident))
                };

                label.render_ref(cols[0], buf);
                ::reformy::ratatui::widgets::Widget::render(self.#ident.input.widget(), cols[1], buf);
            }
        };
        StructField {
            field: ident.clone(),
            field_ty: FieldType {
                ty: parse2(quote! {::reformy::Filtext::<#ty>}).unwrap(),
                is_leaf: true,
            },
            build: to_fields,
            render,
            needs_validation: is_numeric_type(ty),
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

fn is_numeric_type(ty: &syn::Type) -> bool {
    let ty_str = ty.to_token_stream().to_string().replace(' ', "");
    matches!(
        ty_str.as_str(),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "f32" | "f64"
    )
}

/// Convert snake_case function name to PascalCase struct name
fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

/// Attribute macro for generating forms from function parameters
/// Supports #[form(nested)] on parameters for nested Form types
#[proc_macro_attribute]
pub fn reformy_cmd(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;
    let fn_sig = &input.sig;
    let fn_block = &input.block;
    
    // Generate struct name from function name (snake_case -> PascalCase)
    let struct_name = format_ident!("{}", snake_to_pascal(&fn_name.to_string()));
    
    // Extract function parameters with their attributes
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();
    let mut param_attrs = Vec::new();
    let mut new_fn_inputs = Vec::new();
    
    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                param_names.push(pat_ident.ident.clone());
                param_types.push((*pat_type.ty).clone());
                param_attrs.push(pat_type.attrs.clone());
                
                // Create cleaned function input without form attributes
                let mut new_pat_type = pat_type.clone();
                new_pat_type.attrs.retain(|attr| !attr.path().is_ident("form"));
                new_fn_inputs.push(FnArg::Typed(new_pat_type));
            }
        }
    }
    
    let fn_output = &input.sig.output;
    let has_params = !param_names.is_empty();
    
    let fn_generics = &input.sig.generics;
    let fn_asyncness = &input.sig.asyncness;
    let fn_unsafety = &input.sig.unsafety;
    let fn_abi = &input.sig.abi;
    
    let expanded = if has_params {
        // Function has parameters - generate form
        quote! {
            // Keep the original function (without form attributes on params)
            #(#fn_attrs)*
            #fn_vis #fn_asyncness #fn_unsafety #fn_abi fn #fn_name #fn_generics(#(#new_fn_inputs),*) #fn_output {
                #fn_block
            }
            
            // Generate the Args struct with Form
            #[derive(Debug, Default, ::reformy::Form)]
            #fn_vis struct #struct_name {
                #(
                    #(#param_attrs)*
                    pub #param_names: #param_types,
                )*
            }
            
            impl #struct_name {
                /// Execute the function with these arguments
                pub fn execute(self) #fn_output {
                    #fn_name(#(self.#param_names),*)
                }
                
                pub const HAS_PARAMS: bool = true;
            }
        }
    } else {
        // Zero-arg function - no form needed
        let form_name = format_ident!("{}Form", struct_name);
        quote! {
            // Keep the original function
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                #fn_block
            }
            
            // Generate empty struct (no Form derive)
            #[derive(Debug, Default)]
            #fn_vis struct #struct_name;
            
            // Generate dummy form type (won't be used but needs to exist)
            #[derive(Debug)]
            #fn_vis struct #form_name;
            
            impl #form_name {
                pub fn new() -> Self { Self }
                
                pub fn input(&mut self, _input: ::reformy::tui_textarea::Input) -> bool {
                    false
                }
                
                pub fn build(&self) -> Option<#struct_name> {
                    Some(#struct_name::default())
                }
            }
            
            impl ::reformy::ratatui::widgets::StatefulWidgetRef for #form_name {
                type State = bool;
                fn render_ref(&self, _area: ::reformy::ratatui::layout::Rect, _buf: &mut ::reformy::ratatui::buffer::Buffer, _state: &mut Self::State) {
                }
            }
            
            impl #struct_name {
                /// Execute the function with these arguments
                pub fn execute(self) #fn_output {
                    #fn_name()
                }
                
                pub const HAS_PARAMS: bool = false;
                
                pub fn form() -> #form_name {
                    #form_name::new()
                }
            }
        }
    };
    
    expanded.into()
}

// Menu item structure - either a command or a category with submenus
#[derive(Clone)]
enum MenuItem {
    Command(syn::Ident),
    Category {
        name: String,
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    fn collect_commands(&self, commands: &mut Vec<syn::Ident>) {
        match self {
            MenuItem::Command(ident) => commands.push(ident.clone()),
            MenuItem::Category { items, .. } => {
                for item in items {
                    item.collect_commands(commands);
                }
            }
        }
    }
}

// Custom parser for menu items
fn parse_menu_items(input: syn::parse::ParseStream) -> syn::Result<Vec<MenuItem>> {
    let mut items = Vec::new();
    
    while !input.is_empty() {
        // Try to parse a string literal (category)
        if input.peek(syn::LitStr) {
            let category_name: syn::LitStr = input.parse()?;
            input.parse::<syn::Token![=>]>()?;
            
            let content;
            syn::braced!(content in input);
            let subitems = parse_menu_items(&content)?;
            
            items.push(MenuItem::Category {
                name: category_name.value(),
                items: subitems,
            });
        } else {
            // Parse a command identifier
            let ident: syn::Ident = input.parse()?;
            items.push(MenuItem::Command(ident));
        }
        
        // Optional trailing comma
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
    }
    
    Ok(items)
}

/// Collection macro that generates a complete TUI for multiple commands
#[proc_macro]
pub fn reformy_commands(input: TokenStream) -> TokenStream {
    // Parse menu items (flat or nested)
    let menu_items = parse_macro_input!(input with parse_menu_items);
    
    // Collect all commands (flattened from any nesting level)
    let mut commands = Vec::new();
    for item in &menu_items {
        item.collect_commands(&mut commands);
    }
    
    // Generate struct names for each command (snake_case -> PascalCase)
    let struct_names: Vec<_> = commands.iter().map(|cmd| {
        format_ident!("{}", snake_to_pascal(&cmd.to_string()))
    }).collect();
    
    let form_names: Vec<_> = struct_names.iter().map(|name| {
        format_ident!("{}Form", name)
    }).collect();
    
    let command_names: Vec<_> = commands.iter().map(|cmd| cmd.to_string()).collect();
    let num_commands = commands.len();
    
    // Generate indices for match arms
    let indices: Vec<_> = (0..num_commands).collect();
    
    // Generate the runtime menu structure
    fn generate_menu_structure(items: &[MenuItem]) -> proc_macro2::TokenStream {
        let mut item_tokens = Vec::new();
        
        for item in items {
            let token = match item {
                MenuItem::Command(ident) => {
                    let name = ident.to_string();
                    quote! { RuntimeMenuItem::Command(#name) }
                }
                MenuItem::Category { name, items: subitems } => {
                    let subitems_tokens = generate_menu_structure(subitems);
                    quote! {
                        RuntimeMenuItem::Category {
                            name: #name.to_string(),
                            items: vec![#subitems_tokens],
                        }
                    }
                }
            };
            item_tokens.push(token);
        }
        
        quote! { #(#item_tokens),* }
    }
    
    let menu_structure = generate_menu_structure(&menu_items);
    
    let expanded = quote! {
        {
            use ::reformy::ratatui::layout::{Layout, Direction, Constraint};
            use ::reformy::ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget, WidgetRef};
            use ::reformy::ratatui::style::{Style, Color, Modifier};
            use ::reformy::ratatui::text::Line;
            
            // Runtime menu structure
            #[derive(Clone)]
            enum RuntimeMenuItem {
                Command(&'static str),
                Category {
                    name: String,
                    items: Vec<RuntimeMenuItem>,
                },
            }
            
            // Enum to hold any of the command forms
            enum CommandFormState {
                #(#struct_names(#form_names),)*
            }
            
            impl CommandFormState {
                fn input(&mut self, input: ::reformy::tui_textarea::Input) -> bool {
                    match self {
                        #(CommandFormState::#struct_names(form) => form.input(input),)*
                    }
                }
                
                fn render(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer) {
                    match self {
                        #(CommandFormState::#struct_names(form) => {
                            ::reformy::ratatui::widgets::StatefulWidgetRef::render_ref(form, area, buf, &mut true)
                        },)*
                    }
                }
            }
            
            // Application state
            struct AppState {
                menu_items: Vec<RuntimeMenuItem>,
                menu_path: Vec<usize>,  // Path through nested menus
                selected_index: usize,   // Index in current menu level
                current_form: Option<CommandFormState>,
                result: Option<String>,
            }
            
            impl AppState {
                fn new(menu_items: Vec<RuntimeMenuItem>) -> Self {
                    Self {
                        menu_items,
                        menu_path: Vec::new(),
                        selected_index: 0,
                        current_form: None,
                        result: None,
                    }
                }
                
                // Get current menu level based on path
                fn current_menu(&self) -> &[RuntimeMenuItem] {
                    let mut current = &self.menu_items[..];
                    for &index in &self.menu_path {
                        if let RuntimeMenuItem::Category { items, .. } = &current[index] {
                            current = items;
                        }
                    }
                    current
                }
                
                // Get breadcrumb path
                fn breadcrumb(&self) -> Vec<String> {
                    let mut breadcrumb = Vec::new();
                    let mut current = &self.menu_items[..];
                    
                    for &index in &self.menu_path {
                        if let RuntimeMenuItem::Category { name, items } = &current[index] {
                            breadcrumb.push(name.clone());
                            current = items;
                        }
                    }
                    
                    breadcrumb
                }
                
                // Find command name from any level
                fn find_command_name(&self, cmd_name: &str) -> Option<usize> {
                    let command_names = &[#(#command_names),*];
                    command_names.iter().position(|&name| name == cmd_name)
                }
                
                fn handle_input(&mut self, input: ::reformy::tui_textarea::Input) -> bool {
                    use ::reformy::tui_textarea::Key;
                    
                    // If showing result, any key goes back to menu
                    if self.result.is_some() {
                        self.result = None;
                        self.current_form = None;
                        return true;
                    }
                    
                    // If in form view
                    if let Some(form) = &mut self.current_form {
                        match input.key {
                            Key::Esc => {
                                self.current_form = None;
                                return true;
                            }
                            Key::Enter => {
                                // Try to execute the command
                                let result = self.execute_current();
                                if let Some(r) = result {
                                    self.result = Some(r);
                                }
                                return true;
                            }
                            _ => {
                                return form.input(input);
                            }
                        }
                    }
                    
                    // Menu navigation
                    let current_menu = self.current_menu();
                    let menu_len = current_menu.len();
                    
                    match input.key {
                        Key::Down | Key::Char('j') => {
                            self.selected_index = if self.selected_index + 1 >= menu_len {
                                0  // Wrap to top
                            } else {
                                self.selected_index + 1
                            };
                            true
                        }
                        Key::Up | Key::Char('k') => {
                            self.selected_index = if self.selected_index == 0 {
                                menu_len - 1  // Wrap to bottom
                            } else {
                                self.selected_index - 1
                            };
                            true
                        }
                        Key::PageDown | Key::End => {
                            self.selected_index = menu_len - 1;
                            true
                        }
                        Key::PageUp | Key::Home => {
                            self.selected_index = 0;
                            true
                        }
                        Key::Char('G') => {
                            self.selected_index = menu_len - 1;
                            true
                        }
                        Key::Char('g') => {
                            self.selected_index = 0;
                            true
                        }
                        Key::Backspace | Key::Char('h') if !self.menu_path.is_empty() => {
                            // Go back up one level
                            self.menu_path.pop();
                            self.selected_index = 0;
                            true
                        }
                        Key::Enter | Key::Char('l') => {
                            // Enter category or execute command
                            match &current_menu[self.selected_index] {
                                RuntimeMenuItem::Category { .. } => {
                                    // Enter this category
                                    self.menu_path.push(self.selected_index);
                                    self.selected_index = 0;
                                    true
                                }
                                RuntimeMenuItem::Command(cmd_name) => {
                                    // Execute this command
                                    if let Some(cmd_idx) = self.find_command_name(cmd_name) {
                                        match cmd_idx {
                                            #(#indices => {
                                                if #struct_names::HAS_PARAMS {
                                                    self.current_form = Some(CommandFormState::#struct_names(#struct_names::form()));
                                                } else {
                                                    let args = #struct_names::default();
                                                    let result = args.execute();
                                                    self.result = Some(format!("{:?}", result));
                                                }
                                            },)*
                                            _ => return false,
                                        }
                                    }
                                    true
                                }
                            }
                        }
                        _ => false,
                    }
                }
                
                fn execute_current(&self) -> Option<String> {
                    match &self.current_form {
                        #(Some(CommandFormState::#struct_names(form)) => {
                            form.build().map(|args| {
                                let result = args.execute();
                                format!("{:?}", result)
                            })
                        },)*
                        None => None,
                    }
                }
                
                fn render(&self, area: ::reformy::ratatui::layout::Rect, buf: &mut ::reformy::ratatui::buffer::Buffer) {
                    // If showing result
                    if let Some(result) = &self.result {
                        let block = Block::default()
                            .title("Result (press any key to continue)")
                            .borders(Borders::ALL);
                        let inner = block.inner(area);
                        block.render(area, buf);
                        
                        Paragraph::new(result.as_str()).render(inner, buf);
                        return;
                    }
                    
                    // If in form view
                    if let Some(form) = &self.current_form {
                        let block = Block::default()
                            .title("Enter Parameters (Enter to execute, Esc to cancel)")
                            .borders(Borders::ALL);
                        let inner = block.inner(area);
                        block.render(area, buf);
                        
                        form.render(inner, buf);
                        return;
                    }
                    
                    // Build breadcrumb and title
                    let breadcrumb = self.breadcrumb();
                    let breadcrumb_str = if breadcrumb.is_empty() {
                        "Commands".to_string()
                    } else {
                        format!("{} >", breadcrumb.join(" > "))
                    };
                    
                    let title = if self.menu_path.is_empty() {
                        format!("{} (Enter/l: select, Esc: quit, h: back)", breadcrumb_str)
                    } else {
                        format!("{} (Enter/l: select, h/Backspace: back)", breadcrumb_str)
                    };
                    
                    let block = Block::default()
                        .title(title)
                        .borders(Borders::ALL);
                    let inner = block.inner(area);
                    block.render(area, buf);
                    
                    // Get current menu items
                    let current_menu = self.current_menu();
                    let items: Vec<ListItem> = current_menu
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            let (name, is_category) = match item {
                                RuntimeMenuItem::Command(name) => (*name, false),
                                RuntimeMenuItem::Category { name, .. } => (name.as_str(), true),
                            };
                            
                            let style = if idx == self.selected_index {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            };
                            
                            let prefix = if idx == self.selected_index { "> " } else { "  " };
                            let suffix = if is_category { " →" } else { "" };
                            
                            ListItem::new(Line::from(format!("{}{}{}", prefix, name, suffix))).style(style)
                        })
                        .collect();
                    
                    let list = List::new(items);
                    list.render(inner, buf);
                }
            }
            
            // Build menu structure
            let menu_items = vec![#menu_structure];
            
            // Run the TUI
            let mut app = AppState::new(menu_items);
            let mut terminal = ::reformy::ratatui::init();
            
            loop {
                terminal.draw(|f| {
                    app.render(f.area(), f.buffer_mut());
                }).unwrap();
                
                if let ::reformy::crossterm::event::Event::Key(key) = ::reformy::crossterm::event::read().unwrap() {
                    match key.code {
                        ::reformy::crossterm::event::KeyCode::Esc if app.current_form.is_none() && app.result.is_none() && app.menu_path.is_empty() => break,
                        key_code => {
                            let input = ::reformy::tui_textarea::Input {
                                key: key_code.into(),
                                ctrl: key.modifiers.contains(::reformy::crossterm::event::KeyModifiers::CONTROL),
                                alt: key.modifiers.contains(::reformy::crossterm::event::KeyModifiers::ALT),
                                shift: key.modifiers.contains(::reformy::crossterm::event::KeyModifiers::SHIFT),
                            };
                            app.handle_input(input);
                        }
                    }
                }
            }
            
            ::reformy::ratatui::restore();
        }
    };
    
    expanded.into()
}
