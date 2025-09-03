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
                }
                Ok(())
            })?;
        }
    }

    let type_name_str = name.to_string();

    let expanded = quote! {
        impl #impl_generics bevy_persist::Persistable for #name #ty_generics #where_clause {
            fn type_name() -> &'static str {
                #type_name_str
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
                auto_save: #auto_save,
                register_fn: |app: &mut bevy::prelude::App| {
                    bevy_persist::register_persist_type::<#name #ty_generics>(app, #auto_save);
                },
            }
        }
    };

    Ok(expanded)
}
