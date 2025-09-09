extern crate proc_macro;

use fixlite::type_check::{is_str_ref, IsTypeCompatible};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields, GenericParam, Ident,
    Lifetime, Lit, Type, WherePredicate,
};

#[proc_macro_derive(FixDeserialize, attributes(fix, fix_group, fix_registry))]
pub fn fix_deserialize_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let registry_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("fix_registry"))
        .map(|attr| {
            let ident = attr.parse_args::<Ident>().unwrap();
            quote! {#ident}
        })
        .unwrap_or_else(|| {
            quote! {::fixlite::fix::tag::DefaultRegistry}
        });
    let mut assertions = Vec::new();

    // Get the struct name.
    let struct_name = input.ident;

    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut impl_generics_new = generics.clone();
    let fix_lifetime: Lifetime = parse_quote!('fixlite);
    let fix_lifetime_def = GenericParam::Lifetime(parse_quote!(#fix_lifetime));
    impl_generics_new.params.insert(0, fix_lifetime_def);

    let mut where_clause_new = impl_generics_new.where_clause.clone();
    if where_clause_new.is_none() {
        where_clause_new = Some(parse_quote!(where));
    }
    if let Some(ref mut where_clause_new) = where_clause_new {
        // For each struct lifetime 'a, add a where clause: 'fixlite: 'a
        for lt in generics.lifetimes() {
            let lt_ident = &lt.lifetime;
            let predicate: WherePredicate = parse_quote!('fixlite: #lt_ident);
            where_clause_new.predicates.push(predicate);
        }
    }
    impl_generics_new.where_clause = where_clause_new;
    let user_lifetime_defs: Vec<_> = impl_generics_new
        .lifetimes()
        .skip(1)
        .map(|lt| lt.lifetime.clone())
        .collect();
    assert!(
        !user_lifetime_defs.contains(&fix_lifetime),
        "The lifetime `'fixlite` is reserved by fixlite. Please rename it in your struct."
    );

    let (impl_generics_new, _, where_clause_new) = impl_generics_new.split_for_impl();
    let fix_lifetime: proc_macro2::TokenStream = quote!('fixlite);
    let user_lifetime_tokens = if user_lifetime_defs.is_empty() {
        quote!() // no lifetimes, omit <...>
    } else {
        quote! { <#(#user_lifetime_defs),*> }
    };

    // Generate code based on the struct's data.
    let impl_block = match input.data {
        Data::Struct(ref data_struct) => {
            // Process the struct fields.
            let mut field_parsers = Vec::new();
            let mut field_initializers = Vec::new();
            let mut field_names = Vec::new();
            let mut field_checks = Vec::new();
            let mut known_tags: Vec<u32> = Vec::new(); // Collect known tags
            let mut component_handlers = Vec::new();

            if let Fields::Named(ref fields_named) = data_struct.fields {
                for field in &fields_named.named {
                    let field_name = field.ident.as_ref().unwrap();
                    field_names.push(field_name.clone());

                    let field_type = &field.ty;

                    let mut is_component = false;
                    let mut is_group = false;
                    let mut tag_value = None;

                    for attr in &field.attrs {
                        if attr.path().is_ident("fix") {
                            // #[fix(...)]
                            let (comp, tag_opt) = parse_fix_attribute(attr);
                            is_component = comp;

                            if let Some(tag) = tag_opt {
                                tag_value = Some(tag.clone());

                                // <-- put it into the constant list *unless* this field is a component
                                if !is_component {
                                    known_tags.push(tag.parse().unwrap());
                                }
                                if let Ok(tag_u32) = tag.parse::<u32>() {
                                    if let Some(inner_type) =
                                        extract_inner_type(field_type, "Option")
                                    {
                                        assertions.push(quote! { assert_allowed::<Option<#inner_type>, #tag_u32>(); });
                                        assertions.push(
                                            quote! { assert_allowed::<#inner_type, #tag_u32>(); },
                                        );
                                    } else {
                                        assertions.push(
                                            quote! { assert_allowed::<#field_type, #tag_u32>(); },
                                        );
                                    }
                                }
                            }
                        } else if attr.path().is_ident("fix_group") {
                            // #[fix_group(tag = ...)]
                            is_group = true;
                            let (_, tag_opt) = parse_fix_attribute(attr);

                            let tag = tag_opt.expect("group tag must be specified");
                            tag_value = Some(tag.clone());
                            known_tags.push(::fixlite::parse_u32(tag.as_bytes()).unwrap()); // group-counter tags belong to the outer struct
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

                    /* ---------- decide what kind of parser to inject ---------- */
                    if is_component {
                        // component
                        let handler =
                            generate_component_parser(field_name, field_type, &fix_lifetime);
                        component_handlers.push(handler);

                        // ⬇ NEW: we still need the field-check!
                        field_checks.push(generate_field_check(field_name, field_type));
                    } else if let Some(tag) = tag_value {
                        // regular field or repeating-group
                        if is_group {
                            field_parsers.push(generate_group_parser(
                                field_name,
                                field_type,
                                tag,
                                &fix_lifetime,
                            ));
                        } else {
                            // Generate code for regular field parsing.
                            let parser = generate_field_parser(field_name, field_type, tag);
                            field_parsers.push(parser);
                        }
                        field_checks.push(generate_field_check(field_name, field_type));
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

                    fn deserialize_fields<I, F>(
                        fields: &mut I,
                        is_a_top_level_tag: F,
                    ) -> Result<Self, #fix_module_path::FixError>
                    where
                        I: Iterator<Item = &#fix_lifetime str>,
                        F: Fn(u32) -> bool,
                    {
                        use chrono::{NaiveDateTime, DateTime, Utc};
                        let mut first_tag = None;
                        #(#field_initializers)*

                        while let Some(tag) = fields.next() {
                            if tag.is_empty() {
                                break;
                            }
                            let tag: u32 = ::fixlite::parse_u32(tag.as_bytes()).unwrap();

                            // ---------- REPEATING GROUPS ----------
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

                            // ---------- COMPONENTS ----------
                            #(#component_handlers)*         // <-- executes all generated `if … continue;` blocks

                            match tag {
                                #(#field_parsers)*
                                _ => {
                                    // Unrecognized tag, consume and ignore.
                                    fields.next().ok_or(::fixlite::FixError::InvalidValue(0))?;
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

                    fn is_known_tag(tag: u32) -> bool {
                        const KNOWN_TAGS: [u32; #known_tags_len] = [#(#known_tags_tokens),*];
                        KNOWN_TAGS.binary_search(&tag).is_ok()
                    }
                }
            }
        }
        _ => unimplemented!("FixDeserialize can only be derived for structs with named fields."),
    };
    let const_assert_fn_name = format_ident!(
        "_ASSERT_ALLOWED_TAG_TYPES_{}",
        struct_name.to_string().to_uppercase()
    );
    let const_block = quote! {
        const #const_assert_fn_name: () = {
            const fn assert_allowed<T, const TAG: u32>()
            where
                #registry_type: ::fixlite::fix::tag::AllowedType<TAG,T>,
            {}
            fn __assertions #user_lifetime_tokens () {
                #(#assertions)*
            }
            let _ = __assertions;
        };
    };

    // Convert the generated code into a TokenStream.
    TokenStream::from(quote! {
        #impl_block
        #const_block
    })
}

/// Returns (is_component, tag_value)
fn parse_fix_attribute(attr: &Attribute) -> (bool, Option<String>) {
    let mut tag = None;
    let mut is_component = false;

    let _ = attr
        .parse_nested_meta(|nested| {
            if nested.path.is_ident("component") {
                is_component = true;
            } else if nested.path.is_ident("tag") {
                nested.value()?.parse::<Lit>().map(|lit| match lit {
                    Lit::Str(s) => tag = Some(s.value()),
                    Lit::Int(i) => tag = Some(i.base10_digits().to_string()),
                    _ => {}
                })?;
            }
            Ok(())
        })
        .is_ok();

    (is_component, tag)
}

fn generate_field_parser(
    field_name: &Ident,
    field_type: &Type,
    tag: String,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);

    let tag: u32 = ::fixlite::parse_u32(tag.as_bytes()).unwrap();

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
        quote! { value.parse::<#parse_into_type>().map_err(|_| ::fixlite::FixError::InvalidValue(0))? }
    };

    quote! {
        #tag => {
            let value = fields.next().unwrap();
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

    let tag: u32 = ::fixlite::parse_u32(tag.as_bytes()).unwrap();

    // Extract inner type from Vec<T>
    let inner_type =
        extract_inner_type(field_type, "Vec").expect("Expected Vec<T> for repeating group");

    quote! {
        #tag => {
            let value = fields.next().unwrap();
            let group_count = value.parse::<usize>().map_err(|_| ::fixlite::FixError::InvalidValue(0))?;
            let mut entries = Vec::with_capacity(group_count);
            for _ in 0..group_count {
                let entry = <#inner_type as ::fixlite::FixDeserialize<#fix_lifetime>>::deserialize_fields(fields, |tag| Self::is_known_tag(tag))?;
                entries.push(entry);
            }
            #field_var = Some(entries);
        },
    }
}
fn generate_component_parser(
    field_name: &Ident,
    field_type: &Type,
    fix_lifetime: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let field_var = format_ident!("{}_tmp", field_name);
    let inner_type = extract_inner_type(field_type, "Option").unwrap_or(field_type);

    quote! {
        if #field_var.is_none()
           && <#inner_type as ::fixlite::FixDeserialize<#fix_lifetime>>::is_known_tag(tag)
        {
            let value =
                <#inner_type as ::fixlite::FixDeserialize<#fix_lifetime>>
                    ::deserialize_fields(fields, |t| Self::is_known_tag(t))?;
            #field_var = Some(value);
            continue;           // we already consumed the component’s fields
        }
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
/// Used to extract inner type T from  Option<T>, Vec<T>, etc.
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
