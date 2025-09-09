use syn::{
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypePath, parse_str,
};

fn is_syn_type_compatible(expected_type: &str, syn_type: &Type) -> bool {
    // Parse the type name into a syn::Type
    if let Ok(expected_type) = parse_str::<Type>(expected_type) {
        compare_types(&expected_type, syn_type)
    } else {
        false
    }
}

#[cfg(test)]
fn is_type_compatible_ty<T>(syn_type: &Type) -> bool {
    is_syn_type_compatible(std::any::type_name::<T>(), syn_type)
}

#[cfg(test)]
fn is_type_compatible<T>(type_str: &str) -> bool {
    let ty = parse_str::<Type>(type_str).unwrap();
    is_type_compatible_ty::<T>(&ty)
}

pub trait IsTypeCompatible {
    fn is_type_compatible(&self, syn_type: &Type) -> bool;
}
impl IsTypeCompatible for str {
    /// Checks if the type `syn_type` is compatible with the string representation of a type.
    /// Compatibility here means that `syn_type` is likely the same type as `self`.
    /// If this function returns `false`, it indicates that `syn_type` is definitely not
    /// representing the same type as `self`.
    fn is_type_compatible(&self, syn_type: &Type) -> bool {
        is_syn_type_compatible(self, syn_type)
    }
}

fn compare_types(a: &Type, b: &Type) -> bool {
    match (a, b) {
        // Compare TypePath variants
        (Type::Path(a_path), Type::Path(b_path)) => compare_paths(a_path, b_path),
        // Compare TypeReference variants (e.g., &T)
        (Type::Reference(a_ref), Type::Reference(b_ref)) => compare_types(&a_ref.elem, &b_ref.elem),
        // Handle other types as needed
        _ => false,
    }
}

fn compare_paths(a: &TypePath, b: &TypePath) -> bool {
    // Get the last segments of the paths
    let a_segment = a.path.segments.last();
    let b_segment = b.path.segments.last();

    match (a_segment, b_segment) {
        (Some(a_seg), Some(b_seg)) => {
            // Compare the type names (idents)
            if a_seg.ident != b_seg.ident {
                return false;
            }
            // Compare generic arguments if any
            match (&a_seg.arguments, &b_seg.arguments) {
                (PathArguments::AngleBracketed(a_args), PathArguments::AngleBracketed(b_args)) => {
                    compare_generic_arguments(a_args, b_args)
                }
                (PathArguments::None, PathArguments::None) => true,
                _ => false,
            }
        }
        _ => false,
    }
}

fn compare_generic_arguments(
    a: &AngleBracketedGenericArguments,
    b: &AngleBracketedGenericArguments,
) -> bool {
    if a.args.len() != b.args.len() {
        return false;
    }
    for (a_arg, b_arg) in a.args.iter().zip(b.args.iter()) {
        match (a_arg, b_arg) {
            (GenericArgument::Type(a_ty), GenericArgument::Type(b_ty)) => {
                if !compare_types(a_ty, b_ty) {
                    return false;
                }
            }
            _ => continue, // Ignore lifetimes and consts for simplicity
        }
    }
    true
}

pub fn is_str_ref(ty: &Type) -> bool {
    if let Type::Reference(type_ref) = ty
        && let Type::Path(type_path) = &*type_ref.elem
        && let Some(segment) = type_path.path.segments.first()
    {
        return segment.ident == "str";
    }
    false
}

#[cfg(test)]
mod test {
    use crate::type_check::{is_str_ref, is_type_compatible};
    use chrono::{DateTime, Utc};
    use syn::Type;

    #[test]
    fn test_is_str_ref() {
        assert!(is_str_ref(&syn::parse_str::<Type>("&str").unwrap()));
        assert!(is_str_ref(&syn::parse_str::<Type>("&'a str").unwrap()));
        assert!(!is_str_ref(&syn::parse_str::<Type>("str").unwrap()));
        assert!(!is_str_ref(&syn::parse_str::<Type>("String").unwrap()));
        assert!(!is_str_ref(&syn::parse_str::<Type>("&String").unwrap()));
    }

    #[test]
    fn test_is_type_compatible() {
        // Test cases as per your requirements
        assert!(is_type_compatible::<u32>("u32"));
        assert!(is_type_compatible::<Vec<u32>>("Vec<u32>"));
        assert!(is_type_compatible::<&Vec<&u32>>("&Vec<&u32>"));
        assert!(is_type_compatible::<Vec<u32>>("alloc::vec::Vec<u32>"));
        assert!(is_type_compatible::<&str>("&str"));
        assert!(is_type_compatible::<DateTime<Utc>>("DateTime<Utc>"));
        assert!(is_type_compatible::<DateTime<Utc>>(
            "chrono::datetime::DateTime<chrono::offset::utc::Utc>"
        ));
        assert!(is_type_compatible::<DateTime<Utc>>(
            "chrono::DateTime<chrono::Utc>"
        ));

        println!("All tests passed!");
    }
}
