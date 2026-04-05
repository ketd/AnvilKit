//! Derive macro for the `Describe` trait.
//!
//! # Usage
//!
//! ```rust,ignore
//! use anvilkit_describe::Describe;
//!
//! #[derive(Describe)]
//! struct BloomSettings {
//!     /// Whether bloom is enabled.
//!     #[describe(hint = "Toggle bloom post-processing")]
//!     pub enabled: bool,
//!
//!     /// HDR brightness threshold for bloom extraction.
//!     #[describe(range = "0.0..5.0", hint = "Higher = less bloom")]
//!     pub threshold: f32,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit, Meta, Expr};

#[proc_macro_derive(Describe, attributes(describe))]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract doc comment from the type
    let type_doc = extract_doc_comment(&input.attrs);

    let fields_code = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_schemas: Vec<_> = fields.named.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap().to_string();
                    let field_type = type_to_string(&f.ty);
                    let field_doc = extract_doc_comment(&f.attrs);
                    let attrs = parse_describe_attrs(&f.attrs);

                    let hint = attrs.hint.unwrap_or_default();
                    let range_min = attrs.range_min.unwrap_or_default();
                    let range_max = attrs.range_max.unwrap_or_default();
                    let default_val = attrs.default.unwrap_or_default();

                    quote! {
                        anvilkit_describe::FieldSchema {
                            name: #field_name,
                            type_name: #field_type,
                            description: #field_doc,
                            hint: #hint,
                            default: #default_val,
                            range_min: #range_min,
                            range_max: #range_max,
                        }
                    }
                }).collect();

                quote! { vec![#(#field_schemas),*] }
            }
            Fields::Unnamed(_) => quote! { vec![] },
            Fields::Unit => quote! { vec![] },
        },
        Data::Enum(data) => {
            let variant_schemas: Vec<_> = data.variants.iter().map(|v| {
                let variant_name = v.ident.to_string();
                let variant_doc = extract_doc_comment(&v.attrs);
                quote! {
                    anvilkit_describe::FieldSchema {
                        name: #variant_name,
                        type_name: "variant",
                        description: #variant_doc,
                        hint: "",
                        default: "",
                        range_min: "",
                        range_max: "",
                    }
                }
            }).collect();
            quote! { vec![#(#variant_schemas),*] }
        }
        Data::Union(_) => quote! { vec![] },
    };

    let expanded = quote! {
        impl anvilkit_describe::Describe for #name {
            fn schema() -> anvilkit_describe::ComponentSchema {
                anvilkit_describe::ComponentSchema {
                    name: #name_str,
                    description: #type_doc,
                    fields: #fields_code,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    let mut docs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(lit) = &expr_lit.lit {
                        docs.push(lit.value().trim().to_string());
                    }
                }
            }
        }
    }
    docs.join(" ")
}

fn type_to_string(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
        .replace(" ", "")
}

struct DescribeAttrs {
    hint: Option<String>,
    range_min: Option<String>,
    range_max: Option<String>,
    default: Option<String>,
}

fn parse_describe_attrs(attrs: &[syn::Attribute]) -> DescribeAttrs {
    let mut result = DescribeAttrs {
        hint: None,
        range_min: None,
        range_max: None,
        default: None,
    };

    for attr in attrs {
        if !attr.path().is_ident("describe") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("hint") {
                let value = meta.value()?;
                let s: Lit = value.parse()?;
                if let Lit::Str(ls) = s {
                    result.hint = Some(ls.value());
                }
            } else if meta.path.is_ident("range") {
                let value = meta.value()?;
                let s: Lit = value.parse()?;
                if let Lit::Str(ls) = s {
                    let range_str = ls.value();
                    if let Some((min, max)) = range_str.split_once("..") {
                        result.range_min = Some(min.trim_end_matches('=').to_string());
                        result.range_max = Some(max.trim_start_matches('=').to_string());
                    }
                }
            } else if meta.path.is_ident("default") {
                let value = meta.value()?;
                let s: Lit = value.parse()?;
                if let Lit::Str(ls) = s {
                    result.default = Some(ls.value());
                }
            }
            Ok(())
        });
    }

    result
}
