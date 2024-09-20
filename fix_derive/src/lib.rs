// fix_deserialize_derive/src/lib.rs

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, MetaNameValue,
    NestedMeta, Type,
};

#[proc_macro_derive(FixDeserialize, attributes(fix, fix_group))]
pub fn fix_deserialize_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name.
    let struct_name = input.ident;

    // Generate code based on the struct's data.
    let expanded = match input.data {
        Data::Struct(ref data_struct) => {
            // Process the struct fields.
            let mut field_parsers = Vec::new();
            let mut field_initializers = Vec::new();
            let mut field_names = Vec::new();
            let mut field_checks = Vec::new();

            if let Fields::Named(ref fields_named) = data_struct.fields {
                for field in &fields_named.named {
                    let field_name = field.ident.as_ref().unwrap();
                    field_names.push(field_name.clone());

                    let field_type = &field.ty;

                    let mut is_group = false;
                    let mut tag_value = None;
                    let mut type_value = None;

                    for attr in &field.attrs {
                        if attr.path.is_ident("fix") {
                            let (tag, ty) = parse_fix_attribute(attr).unwrap();
                            tag_value = Some(tag);
                            type_value = Some(ty);
                        } else if attr.path.is_ident("fix_group") {
                            let tag = parse_fix_group_attribute(attr).unwrap();
                            tag_value = Some(tag);
                            is_group = true;
                        }
                    }

                    // Generate code for field initialization.
                    let field_var = format_ident!("{}_tmp", field_name);
                    field_initializers.push(quote! {
                        let mut #field_var: Option<#field_type> = None;
                    });

                    if let Some(tag) = tag_value {
                        if is_group {
                            // Generate code for repeating group parsing.
                            let group_parser = generate_group_parser(field_name, field_type, tag);
                            field_parsers.push(group_parser);
                        } else {
                            // Generate code for regular field parsing.
                            let parser = generate_field_parser(field_name, field_type, &type_value.unwrap(), tag);
                            field_parsers.push(parser);
                        }

                        // Generate code for field presence check.
                        let check = generate_field_check(field_name, field_type);
                        field_checks.push(check);
                    }
                }
            }

            let fix_deserialize_path = quote!(::fix);

            // Combine all parts into the final implementation.
            quote! {
                impl #fix_deserialize_path::FixDeserialize for #struct_name {
                    fn from_fix_message(fix_message: &[u8]) -> Result<Self, #fix_deserialize_path::FixError> {
                        let fix_message_str = std::str::from_utf8(fix_message)?;
                        let mut fields = fix_message_str.split('|').peekable();

                        Self::from_fix_message_iter(&mut fields)
                    }

                    fn from_fix_message_iter<'a, I>(fields: &mut std::iter::Peekable<I>) -> Result<Self, #fix_deserialize_path::FixError>
                    where
                        I: Iterator<Item = &'a str>,
                    {
                        use chrono::{NaiveDateTime, DateTime, Utc};
                        let mut first_tag = None;
                        #(#field_initializers)*

                        while let Some(field) = fields.peek().map(|x|*x)  {
                            if field.is_empty() {
                                fields.next();
                                continue;
                            }
                            let mut parts = field.splitn(2, '=');
                            let tag = parts.next().unwrap();
                            // Do not consume the iterator yet.
                            println!("FIELD parts: {:?}={:?}",tag, parts.next().unwrap());

                            if first_tag.is_none() {
                                first_tag = Some(tag);
                            } else if tag == first_tag.unwrap() {
                                break;
                            }

                            match tag {
                                #(#field_parsers)*
                                _ => {
                                    // Unrecognized tag, consume and ignore.
                                    fields.next();
                                }
                            }
                        }

                        #(#field_checks)*

                        Ok(Self {
                            #(
                                #field_names,
                            )*
                        })
                    }
                }
            }
        }
        _ => unimplemented!("FixDeserialize can only be derived for structs with named fields."),
    };

    // Convert the generated code into a TokenStream.
    TokenStream::from(expanded)
}

fn parse_fix_attribute(attr: &Attribute) -> Option<(String, String)> {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        let mut tag = None;
        let mut ty = None;
        for nested_meta in meta_list.nested {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                                        ref path,
                                                        lit: Lit::Str(ref lit_str),
                                                        ..
                                                    })) = nested_meta
            {
                if path.is_ident("tag") {
                    tag = Some(lit_str.value());
                } else if path.is_ident("type") {
                    ty = Some(lit_str.value());
                }
            }
        }
        if tag.is_some() && ty.is_some() {
            return Some((tag.unwrap(), ty.unwrap()));
        }
    }
    None
}

fn parse_fix_group_attribute(attr: &Attribute) -> Option<String> {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        for nested_meta in meta_list.nested {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                                        ref path,
                                                        lit: Lit::Str(ref lit_str),
                                                        ..
                                                    })) = nested_meta
            {
                if path.is_ident("tag") {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn generate_field_parser(
    field_name: &Ident,
    field_type: &Type,
    fix_type: &str,
    tag: String,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);
    let parse_value = match fix_type {
        "String" => quote! { value.to_string() },
        "u8" => quote! { value.parse::<u8>().map_err(|_| ::fix::FixError::InvalidValue(#tag.to_string()))? },
        "u32" => quote! { value.parse::<u32>().map_err(|_| ::fix::FixError::InvalidValue(#tag.to_string()))? },
        "f64" => quote! { value.parse::<f64>().map_err(|_| ::fix::FixError::InvalidValue(#tag.to_string()))? },
        "UTC_TIMESTAMP" => quote! {
            {let dt = NaiveDateTime::parse_from_str(value, "%Y%m%d-%H:%M:%S%.f")?;
            DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)}
        },
        _ => unimplemented!("Unsupported type"),
    };

    quote! {
        #tag => {
            let field = fields.next().unwrap();
            let mut parts = field.splitn(2, '=');
            parts.next(); // Skip tag
            let value = parts.next().unwrap();
            #field_var = Some(#parse_value);
        },
    }
}

fn extract_inner_type<'a>(field_type: &'a Type, expected_outer: &str) -> Option<&'a Type> {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == expected_outer {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}
fn generate_group_parser(
    field_name: &Ident,
    field_type: &Type,
    tag: String,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);

    // Extract inner type from Vec<T>
    let inner_type = extract_inner_type(field_type, "Vec").expect("Expected Vec<T> for repeating group");

    quote! {
        #tag => {
            let field = fields.next().unwrap();
            let mut parts = field.splitn(2, '=');
            let tag = parts.next(); // Skip tag
            let value = parts.next().unwrap();
            let group_count = value.parse::<usize>().map_err(|_| ::fix::FixError::InvalidValue(#tag.to_string()))?;
            let mut entries = Vec::with_capacity(group_count);
            for i in 0..group_count {
                let entry = <#inner_type as ::fix::FixDeserialize>::from_fix_message_iter(fields)?;
                entries.push(entry);
            }
            #field_var = Some(entries);
        },
    }
}

fn generate_field_check(field_name: &Ident, field_type: &Type) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);

    // Check if the field is optional (Option<T>)
    let is_option = match field_type {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                segment.ident == "Option"
            } else {
                false
            }
        }
        _ => false,
    };

    if is_option {
        quote! {
            let #field_name = #field_var;
        }
    } else {
        quote! {
            let #field_name = #field_var.ok_or(::fix::FixError::MissingField(stringify!(#field_name)))?;
        }
    }
}
