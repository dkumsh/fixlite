use crate::fix;
use crate::type_check::IsTypeCompatible;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use quote::quote;
use std::any::type_name;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{Type, TypeReference};

#[derive(Debug)]
struct Metadata {
    allowed_types: Vec<String>,
}

pub trait Registry {
    fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String>;
    fn contains(&self, tag: &str) -> bool;

    fn validate_field_type(&self, tag: &str, field_type: &Type) -> Result<(), syn::Error> {
        let allowed_types = self.get_allowed_types_for_tag(tag);

        // Check if the field type matches with one of the allowed types
        if allowed_types
            .iter()
            .any(|allowed_type| allowed_type.as_str().is_type_compatible(field_type))
        {
            return Ok(());
        }

        if let Some(option_inner_type) = extract_inner_type(field_type, "Option") {
            // Check if the inner type T is allowed
            if allowed_types
                .iter()
                .any(|allowed_type| allowed_type.as_str().is_type_compatible(option_inner_type))
            {
                return Ok(());
            }
        }
        let error = syn::Error::new(
            field_type.span(),
            format!(
                "Field type \"{}\" for tag {} does not match any allowed types: {:?}",
                type_to_string_without_lifetimes(field_type),
                tag,
                allowed_types
            ),
        );
        Err(error)
    }
}

#[derive(Debug)]
pub struct DefaultRegistry {
    registry: HashMap<String, Metadata>,
}

macro_rules! tag_metadata {
    ($registry:expr, $tag:expr, $types:expr) => {
        // Always add "&str" and "String" to the allowed types
        let mut all_types: Vec<String> = vec!["&str".to_string(), "String".to_string()];
        let types: Vec<&str> = $types;
        all_types.extend(types.into_iter().map(|s| s.to_string()));

        $registry.insert(
            $tag.into(),
            Metadata {
                allowed_types: all_types,
            },
        );
    };
}
impl DefaultRegistry {
    fn new() -> Self {
        let mut registry = HashMap::new();
        tag_metadata!(registry, "9", vec![type_name::<u32>()]);
        tag_metadata!(registry, "6", vec![type_name::<f64>()]);
        tag_metadata!(registry, "14", vec![type_name::<f64>()]);
        tag_metadata!(registry, "31", vec![type_name::<f64>()]);
        tag_metadata!(registry, "32", vec![type_name::<f64>()]);
        tag_metadata!(registry, "34", vec![type_name::<u32>()]);
        tag_metadata!(registry, "38", vec![type_name::<f64>()]);
        tag_metadata!(registry, "44", vec![type_name::<f64>()]);
        tag_metadata!(registry, "52", vec![type_name::<DateTime<Utc>>()]);
        tag_metadata!(registry, "151", vec![type_name::<f64>()]);
        tag_metadata!(registry, "270", vec![type_name::<f64>()]);
        tag_metadata!(registry, "271", vec![type_name::<f64>()]);
        tag_metadata!(registry, "272", vec![type_name::<DateTime<Utc>>()]);

        // Enums
        tag_metadata!(registry, "35", vec![type_name::<fix::MsgType>()]);
        tag_metadata!(registry, "20", vec![type_name::<fix::ExecTransType>()]);
        tag_metadata!(registry, "21", vec![type_name::<fix::HandlInst>()]);
        tag_metadata!(registry, "22", vec![type_name::<fix::SecurityIDSource>()]);
        tag_metadata!(registry, "39", vec![type_name::<fix::OrdStatus>()]);
        tag_metadata!(registry, "40", vec![type_name::<fix::OrdType>()]);
        tag_metadata!(registry, "54", vec![type_name::<fix::Side>()]);
        tag_metadata!(registry, "59", vec![type_name::<fix::TimeInForce>()]);
        tag_metadata!(registry, "150", vec![type_name::<fix::ExecType>()]);
        tag_metadata!(
            registry,
            "263",
            vec![type_name::<fix::SubscriptionRequestType>()]
        );
        tag_metadata!(registry, "265", vec![type_name::<fix::MDUpdateType>()]);
        tag_metadata!(registry, "269", vec![type_name::<fix::MDEntryType>()]);
        tag_metadata!(registry, "279", vec![type_name::<fix::MDUpdateAction>()]);
        tag_metadata!(registry, "281", vec![type_name::<fix::MDReqRejReason>()]);
        tag_metadata!(
            registry,
            "321",
            vec![type_name::<fix::SecurityRequestType>()]
        );
        tag_metadata!(
            registry,
            "323",
            vec![type_name::<fix::SecurityResponseType>()]
        );

        tag_metadata!(registry, "10", vec![type_name::<u8>()]);
        tag_metadata!(registry, "default", vec![]);
        Self { registry }
    }
}
impl Registry for DefaultRegistry {
    fn get_allowed_types_for_tag(&self, tag: &str) -> Vec<String> {
        self.registry.get(tag).map_or_else(
            || self.registry.get("default").unwrap().allowed_types.clone(), // Return default types if tag not found
            |metadata| metadata.allowed_types.clone(),
        )
    }
    fn contains(&self, tag: &str) -> bool {
        self.registry.contains_key(tag)
    }
}
static REGISTRY: Lazy<DefaultRegistry> = Lazy::new(DefaultRegistry::new);
pub fn get_registry_instance() -> &'static dyn Registry {
    &*REGISTRY
}
struct RemoveLifetimes;
impl VisitMut for RemoveLifetimes {
    // Visit TypeReference to remove lifetimes from reference types (e.g. &'a T -> &T)
    fn visit_type_reference_mut(&mut self, node: &mut TypeReference) {
        node.lifetime = None;
        syn::visit_mut::visit_type_reference_mut(self, node);
    }
}
fn type_to_string_without_lifetimes(ty: &Type) -> String {
    let mut ty = ty.clone();
    RemoveLifetimes.visit_type_mut(&mut ty);
    quote!(#ty).to_string().replace(' ', "")
}

/// Used to extract inner type T from  Option<T>, Vec<T>, etc.
pub fn extract_inner_type<'a>(field_type: &'a Type, expected_outer: &str) -> Option<&'a Type> {
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

#[cfg(test)]
mod test {
    use crate::fix::tag::{get_registry_instance, type_to_string_without_lifetimes};
    use syn::Type;

    #[test]
    fn test_type_to_string() {
        let field_type: Type = syn::parse_str("&'a Option<&'b Vec<&'c str>>").unwrap();
        assert_eq!(
            "&Option<&Vec<&str>>",
            type_to_string_without_lifetimes(&field_type)
        );
    }
    #[test]
    fn test_validate_field_type() {
        let field_type: Type = syn::parse_str("String").unwrap();
        get_registry_instance()
            .validate_field_type("35", &field_type)
            .unwrap();
        let field_type: Type = syn::parse_str("&str").unwrap();
        get_registry_instance()
            .validate_field_type("35", &field_type)
            .unwrap();
        let field_type: Type = syn::parse_str("&'a str").unwrap();
        get_registry_instance()
            .validate_field_type("35", &field_type)
            .unwrap();
    }

    #[test]
    fn test_tag_registry() {
        assert!(get_registry_instance().contains("35"));
        assert!(get_registry_instance().contains("54"));
    }
}
