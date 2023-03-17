//! Helpers to properly handle generic arguments

use syn::{Generics, Ident, Type};

/// Compute the identifiers of type parameters that are only used in the super
/// struct, but not in the sub struct.
pub(crate) fn merge_generics(
    from_type: &Type,
    subtype_generics: &Generics,
) -> Result<Vec<Ident>, syn::Error> {
    let from_tyidents = collect_all_generics(from_type);

    let subtype_tyidents = {
        let mut idents = Vec::new();
        for x in subtype_generics.params.iter() {
            if let syn::GenericParam::Type(syn::TypeParam { ident, .. }) = x {
                idents.push(ident.clone());
            }
        }
        idents
    };

    eprintln!("from_tyidents: {:?}", from_tyidents);
    eprintln!("subtype_tyidents: {:?}", subtype_tyidents);

    for subtype_tyident in subtype_tyidents.iter() {
        if !from_tyidents.contains(subtype_tyident) {
            return Err(syn::Error::new(
                subtype_tyident.span(),
                format!(
                    "Type parameter '{}' is unknown from super type, which {}",
                    subtype_tyident,
                    if from_tyidents.len() == 0 {
                        "uses none".to_string()
                    } else {
                        format!(
                            "only uses the following: {}",
                            from_tyidents
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                ),
            ));
        }
    }

    let mut res = Vec::new();
    for from_tyident in from_tyidents.into_iter() {
        if !subtype_tyidents.contains(&from_tyident) {
            // eprintln!("Type parameter only used in super struct: {}", from_tyident);
            res.push(from_tyident);
        }
    }

    Ok(res)
}

/// Collect the identifiers of all generics/type parameters within a type definition
fn collect_all_generics(ty: &Type) -> Vec<Ident> {
    let mut res = Vec::new();

    match ty {
        // TODO: impl traits
        // TODO: trait bounds
        // TODO: trait object
        Type::Array(syn::TypeArray { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Group(syn::TypeGroup { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Paren(syn::TypeParen { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Path(syn::TypePath { path, .. }) => {
            for segment in path.segments.iter() {
                if let syn::PathArguments::AngleBracketed(genargs) = &segment.arguments {
                    // here's a generic argument
                    for arg in genargs.args.iter() {
                        match arg {
                            // TODO: lifetime
                            syn::GenericArgument::Type(inner_ty) => {
                                res.append(&mut collect_all_generics_from_type_param(inner_ty));
                            }
                            _ => {}
                        }
                    }
                }
            }
            return res;
        }
        Type::Ptr(syn::TypePtr { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Reference(syn::TypeReference { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Slice(syn::TypeSlice { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            for elem in elems.iter() {
                res.append(&mut collect_all_generics(elem))
            }
            return res;
        }
        _ => {
            return res;
        }
    }
}

/// Extract the generics from a Type that was already a type parameter itself.
///
/// Normally, this is just a "T" etc. But if it contains angled brackets itself again,
/// then we descent into those parameters, e.g. on "Vec<T>".
///
/// If there are angled brackets, descent into them, otherwise return the last
/// path segment identifier.
fn collect_all_generics_from_type_param(ty: &Type) -> Vec<Ident> {
    let mut res = Vec::new();

    match ty {
        // TODO: impl traits
        // TODO: trait bounds
        // TODO: trait object
        Type::Array(syn::TypeArray { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Group(syn::TypeGroup { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Paren(syn::TypeParen { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Path(syn::TypePath { path, .. }) => {
            let mut descented = false;
            for segment in path.segments.iter() {
                if let syn::PathArguments::AngleBracketed(genargs) = &segment.arguments {
                    descented = true;
                    // here's a generic argument
                    for arg in genargs.args.iter() {
                        match arg {
                            // TODO: lifetime
                            syn::GenericArgument::Type(inner_ty) => {
                                res.append(&mut collect_all_generics_from_type_param(inner_ty));
                            }
                            _ => {}
                        }
                    }
                }
            }

            if !descented {
                if let Some(last) = path.segments.iter().last() {
                    res.push(last.ident.clone());
                }
            }
            return res;
        }
        Type::Ptr(syn::TypePtr { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Reference(syn::TypeReference { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Slice(syn::TypeSlice { elem, .. }) => {
            return collect_all_generics(elem);
        }
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            for elem in elems.iter() {
                res.append(&mut collect_all_generics(elem))
            }
            return res;
        }
        _ => {
            return res;
        }
    }
}

/// Given a Generics object, return a new one that has its type params replaced
/// with the given ones.
pub(crate) fn set_types(
    generics: &Generics,
    new_idents: impl IntoIterator<Item = Ident>,
) -> Generics {
    let mut generics = generics.clone();

    generics.params.clear();

    for ident in new_idents {
        // eprintln!("adding type {}", &ident);
        generics.params.push(syn::GenericParam::Type(ident.into()));
    }

    generics
}
