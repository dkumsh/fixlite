extern crate proc_macro;

use fixlite::fix::tag::{extract_inner_type, get_registry_instance};
use fixlite::type_check::{is_str_ref, IsTypeCompatible};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields, GenericParam, Ident,
    Lifetime, Lit, Type, WherePredicate,
};

#[proc_macro_derive(FixDeserialize, attributes(fix, fix_group))]
pub fn fix_deserialize_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name.
    let struct_name = input.ident;

    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut impl_generics_new = generics.clone();
    let fix_lifetime: Lifetime = parse_quote!('fix);
    let fix_lifetime_def = GenericParam::Lifetime(parse_quote!(#fix_lifetime));
    impl_generics_new.params.insert(0, fix_lifetime_def);

    let mut where_clause_new = impl_generics_new.where_clause.clone();
    if where_clause_new.is_none() {
        where_clause_new = Some(parse_quote!(where));
    }
    if let Some(ref mut where_clause_new) = where_clause_new {
        // For each struct lifetime 'a, add a where clause: 'fix: 'a
        for lt in generics.lifetimes() {
            let lt_ident = &lt.lifetime;
            let predicate: WherePredicate = parse_quote!('fix: #lt_ident);
            where_clause_new.predicates.push(predicate);
        }
    }
    impl_generics_new.where_clause = where_clause_new;
    let (impl_generics_new, _, where_clause_new) = impl_generics_new.split_for_impl();
    let fix_lifetime: proc_macro2::TokenStream = quote!('fix);

    // Generate code based on the struct's data.
    let expanded = match input.data {
        Data::Struct(ref data_struct) => {
            // Process the struct fields.
            let mut field_parsers = Vec::new();
            let mut field_initializers = Vec::new();
            let mut field_names = Vec::new();
            let mut field_checks = Vec::new();
            let mut known_tags = Vec::new(); // Collect known tags

            if let Fields::Named(ref fields_named) = data_struct.fields {
                for field in &fields_named.named {
                    let field_name = field.ident.as_ref().unwrap();
                    field_names.push(field_name.clone());

                    let field_type = &field.ty;

                    let mut is_group = false;
                    let mut tag_value = None;

                    for attr in &field.attrs {
                        if attr.path().is_ident("fix") || attr.path().is_ident("fix_group") {
                            let tag = parse_fix_attribute(attr).unwrap();
                            tag_value = Some(tag.clone());
                            // mark repeating-group fields when using #[fix_group]
                            if attr.path().is_ident("fix_group") {
                                is_group = true;
                            }
                            known_tags.push(tag); // Collect known tags
                        }
                    }

                    // Generate code for field initialization.
                    let field_var = format_ident!("{}_tmp", field_name);
                    // Extract inner type from Option<T>
                    if let Some(inner_type) = extract_inner_type(field_type, "Option") {
                        // Field type is Option<T>, so use T
                        field_initializers.push(quote! {
                            let mut #field_var: Option<#inner_type> = None;
                        });
                    } else {
                        field_initializers.push(quote! {
                            let mut #field_var: Option<#field_type> = None;
                        });
                    }

                    if let Some(tag) = tag_value {
                        if is_group {
                            // Generate code for repeating group parsing.
                            let group_parser =
                                generate_group_parser(field_name, field_type, tag, &fix_lifetime);
                            field_parsers.push(group_parser);
                        } else {
                            if let Err(e) = get_registry_instance()
                                .validate_field_type(tag.clone().as_str(), field_type)
                            {
                                return e.to_compile_error().into();
                            }

                            // Generate code for regular field parsing.
                            let parser = generate_field_parser(field_name, field_type, tag);
                            field_parsers.push(parser);
                        }

                        // Generate code for field presence check.
                        let check = generate_field_check(field_name, field_type);
                        field_checks.push(check);
                    }
                }
            }
            // Sort the known tags for binary search.
            known_tags.sort();
            let known_tags_len = known_tags.len();
            let known_tags_tokens = known_tags.iter().map(|tag| quote! { #tag });

            let fix_module_path = quote!(::fixlite);

            // Combine all parts into the final implementation.
            quote! {
                impl #impl_generics_new #fix_module_path::FixDeserialize<#fix_lifetime> for #struct_name #ty_generics #where_clause_new {

                    fn from_fix_message_inner<I, F>(
                        fields: &mut std::iter::Peekable<I>,
                        is_a_top_level_tag: F,
                    ) -> Result<Self, #fix_module_path::FixError>
                    where
                        I: Iterator<Item = &#fix_lifetime str>,
                        F: Fn(&str) -> bool,
                    {
                        use chrono::{NaiveDateTime, DateTime, Utc};
                        let mut first_tag = None;
                        #(#field_initializers)*

                        while let Some(field) = fields.peek().map(|x| *x) {
                            if field.is_empty() {
                                fields.next();
                                continue;
                            }
                            let mut parts = field.splitn(2, '=');
                            let tag = parts.next().unwrap();

                            // The following checks heuristically detect the boundaries of elements
                            // within a repeating group and identify the end of the group.
                            //
                            // Check for the beginning of an element:
                            // This approach assumes that all elements in a repeating group start
                            // with the same tag. This assumption is generally reasonable.
                            if first_tag.is_none() {
                                first_tag = Some(tag);
                            } else if tag == first_tag.unwrap() {
                                // `first_tag` is expected to be the first tag in the repeating group.
                                // If encountered again, it marks the start of the next element of the group,
                                // so we stop processing the current element.
                                // If this is a top-level tag, i.e., it is not part of a repeating group,
                                // this logic does not matter as we do not expect any tag to appear
                                // more than once outside a repeating group.
                                break;
                            }

                            // Check for the end of the group:
                            // We assume that if while processing elements of a repeating group,
                            // we encounter a tag which does not belong to the element but is one
                            // of the top-level tags, this signals, that we are likely past the
                            // last element of the group, so we need to stop processing the current
                            // element.
                            if is_a_top_level_tag(tag) {
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

                    fn is_known_tag(tag: &str) -> bool {
                        const KNOWN_TAGS: [&str; #known_tags_len] = [#(#known_tags_tokens),*];
                        KNOWN_TAGS.binary_search(&tag).is_ok()
                    }
                }
            }
        }
        _ => unimplemented!("FixDeserialize can only be derived for structs with named fields."),
    };

    // Convert the generated code into a TokenStream.
    TokenStream::from(expanded)
}

fn parse_fix_attribute(attr: &Attribute) -> Option<String> {
    let mut tag = None;
    let _ = attr
        .parse_nested_meta(|nested| {
            if nested.path.is_ident("tag") {
                // Accept either a string literal or an integer literal
                nested.value()?.parse::<Lit>().map(|lit| {
                    match lit {
                        Lit::Str(lit_str) => {
                            tag = Some(lit_str.value());
                        }
                        Lit::Int(lit_int) => {
                            // e.g. 200 → "200"
                            tag = Some(lit_int.base10_digits().to_string());
                        }
                        _ => { /* ignore other literal kinds */ }
                    }
                })
            } else {
                Ok(())
            }
        })
        .is_ok();
    tag
}

fn generate_field_parser(
    field_name: &Ident,
    field_type: &Type,
    tag: String,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);

    // Determine the actual type to parse into
    let parse_into_type = if let Some(inner_type) = extract_inner_type(field_type, "Option") {
        inner_type
    } else {
        field_type
    };

    let parse_value = if is_str_ref(parse_into_type) {
        quote! { value }
    } else if "String".is_type_compatible(parse_into_type) {
        quote! { value.to_string() }
    } else if "DateTime<Utc>".is_type_compatible(parse_into_type) {
        quote! {
            {let dt = NaiveDateTime::parse_from_str(value, "%Y%m%d-%H:%M:%S%.f")?;
            DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)}
        }
    } else {
        quote! { value.parse::<#parse_into_type>().map_err(|_| ::fixlite::FixError::InvalidValue(#tag))? }
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

fn generate_group_parser(
    field_name: &Ident,
    field_type: &Type,
    tag: String,
    fix_lifetime: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);

    // Extract inner type from Vec<T>
    let inner_type =
        extract_inner_type(field_type, "Vec").expect("Expected Vec<T> for repeating group");

    quote! {
        #tag => {
            let field = fields.next().unwrap();
            let mut parts = field.splitn(2, '=');
            let tag = parts.next(); // Skip tag
            let value = parts.next().unwrap();
            let group_count = value.parse::<usize>().map_err(|_| ::fixlite::FixError::InvalidValue(#tag))?;
            let mut entries = Vec::with_capacity(group_count);
            for _ in 0..group_count {
                let entry = <#inner_type as ::fixlite::FixDeserialize<#fix_lifetime>>::from_fix_message_inner(fields, |tag| Self::is_known_tag(tag))?;
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
            let #field_name = #field_var.ok_or(::fixlite::FixError::MissingField(stringify!(#field_name)))?;
        }
    }
}
