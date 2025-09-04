use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result as SynResult};

#[proc_macro_derive(Persist, attributes(persist))]
pub fn derive_persist(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_persist(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_persist(input: &DeriveInput) -> SynResult<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse persist attributes if any
    let mut auto_save = true;
    let mut persist_file = None;
    let mut persist_mode = "dev".to_string(); // default mode
    let mut embed_file = None;

    for attr in &input.attrs {
        if attr.path().is_ident("persist") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("auto_save") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitBool = meta.input.parse()?;
                    auto_save = lit.value();
                } else if meta.path.is_ident("file") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    persist_file = Some(lit.value());
                } else if meta.path.is_ident("mode") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    persist_mode = lit.value();
                } else if meta.path.is_ident("embed") {
                    // For embedded resources, specify the file to embed
                    if meta.input.peek(syn::Token![=]) {
                        meta.input.parse::<syn::Token![=]>()?;
                        let lit: syn::LitStr = meta.input.parse()?;
                        embed_file = Some(lit.value());
                        persist_mode = "embed".to_string(); // Set mode to embed when file is specified
                    } else {
                        persist_mode = "embed".to_string();
                    }
                } else if meta.path.is_ident("dynamic") {
                    persist_mode = "dynamic".to_string();
                } else if meta.path.is_ident("secure") {
                    persist_mode = "secure".to_string();
                }
                Ok(())
            })?;
        }
    }

    let type_name_str = name.to_string();
    let persist_mode_str = persist_mode.clone();
    
    // Convert embed_file Option<String> to token stream for static context
    let embed_file_tokens = match embed_file.as_ref() {
        Some(path) => quote! { Some(#path) },
        None => quote! { None },
    };

    // Generate embedded data if in embed mode
    // Only include the file in production builds, in dev we load dynamically
    let embedded_data = if persist_mode == "embed" {
        // Use specified file or auto-generate based on type name
        // Auto-generated files are saved in assets/persist/ directory
        // For include_str!, we need a path relative to the source file where the macro is used
        // Most Bevy projects have src/ and assets/ as siblings, so we use ../assets/persist/
        let file_path = embed_file.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| {
                // Check if we need to use CARGO_MANIFEST_DIR (for workspace members)
                // Otherwise use ../assets relative path (typical for single crate projects)
                format!("../assets/persist/{}.ron", type_name_str.to_lowercase().replace("::", "_"))
            });
        quote! {
            #[cfg(feature = "prod")]
            {
                Some(include_str!(#file_path))
            }
            #[cfg(not(feature = "prod"))]
            {
                None
            }
        }
    } else {
        quote! { None }
    };

    let expanded = quote! {
        impl #impl_generics bevy_persist::Persistable for #name #ty_generics #where_clause {
            fn type_name() -> &'static str {
                #type_name_str
            }

            fn persist_mode() -> bevy_persist::PersistMode {
                match #persist_mode_str {
                    "embed" => bevy_persist::PersistMode::Embed,
                    "dynamic" => bevy_persist::PersistMode::Dynamic,
                    "secure" => bevy_persist::PersistMode::Secure,
                    _ => bevy_persist::PersistMode::Dev,
                }
            }

            fn embedded_data() -> Option<&'static str> {
                #embedded_data
            }

            fn to_persist_data(&self) -> bevy_persist::PersistData {
                let mut data = bevy_persist::PersistData::new();
                if let Ok(json_value) = serde_json::to_value(self) {
                    if let serde_json::Value::Object(map) = json_value {
                        for (key, value) in map {
                            data.values.insert(key, value);
                        }
                    }
                }
                data
            }

            fn load_from_persist_data(&mut self, data: &bevy_persist::PersistData) {
                if let Ok(value) = serde_json::to_value(&data.values) {
                    if let Ok(new_self) = serde_json::from_value(value) {
                        *self = new_self;
                    }
                }
            }
        }

        // Auto-register this type when it's used
        bevy_persist::inventory::submit! {
            bevy_persist::PersistRegistration {
                type_name: #type_name_str,
                persist_mode: #persist_mode_str,
                auto_save: #auto_save,
                embed_file: #embed_file_tokens,
                register_fn: |app: &mut bevy::prelude::App| {
                    bevy_persist::register_persist_type::<#name #ty_generics>(app, #auto_save);
                },
            }
        }
    };

    Ok(expanded)
}
