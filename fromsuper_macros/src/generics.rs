//! Helpers to properly handle generic arguments

use syn::{Generics, Ident, Type};

/// Collect the named lifetimes that need to be added to the impl block.
///
/// The result may contain duplicates. The `'static` lifetime is ignored.
/// An error is raised if `'_` (the anonymous lifetime) is found.
pub(crate) fn collect_extra_lifetimes(
    from_type: &Type,
    subtype_generics: &Generics,
) -> Result<Vec<syn::Lifetime>, syn::Error> {
    let from_lifetimes = collect_all_lifetimes(from_type);

    // forbid '_
    for lifetime in from_lifetimes.iter() {
        if lifetime.ident == "_" {
            return Err(syn::Error::new(
                lifetime.span(),
                format!("The anonymous lifetime '_ is not supported."),
            ));
        }
    }

    // ignore 'static
    let from_lifetimes = from_lifetimes
        .into_iter()
        .filter(|x| x.ident != "static")
        .collect::<Vec<_>>();

    let subtype_lifetimes = {
        let mut idents = Vec::new();
        for x in subtype_generics.params.iter() {
            if let syn::GenericParam::Lifetime(syn::LifetimeDef { lifetime, .. }) = x {
                idents.push(lifetime.clone());
            }
        }
        idents
    };

    // eprintln!("from_lifetimesy: {:?}", from_lifetimes);
    // eprintln!("subtype_lifetimes: {:?}", subtype_lifetimes);

    for subtype_tyident in subtype_lifetimes.iter() {
        if !from_lifetimes.contains(subtype_tyident) {
            return Err(syn::Error::new(
                subtype_tyident.span(),
                format!(
                    "Lifetime parameter '{}' is unknown from super type, which {}",
                    subtype_tyident,
                    if from_lifetimes.len() == 0 {
                        "uses none".to_string()
                    } else {
                        format!(
                            "only uses the following: {}",
                            from_lifetimes
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
    for from_lifetime in from_lifetimes.into_iter() {
        if !subtype_lifetimes.contains(&from_lifetime) {
            // eprintln!("Type parameter only used in super struct: {}", from_tyident);
            res.push(from_lifetime);
        }
    }

    Ok(res)
}

/// Collect the identifiers of all lifetime parameters within a type definition
fn collect_all_lifetimes(ty: &Type) -> Vec<syn::Lifetime> {
    let mut res = Vec::new();

    match ty {
        // TODO: impl traits
        // TODO: trait bounds
        // TODO: trait object
        Type::Array(syn::TypeArray { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Group(syn::TypeGroup { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Paren(syn::TypeParen { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Path(syn::TypePath { path, .. }) => {
            for segment in path.segments.iter() {
                if let syn::PathArguments::AngleBracketed(genargs) = &segment.arguments {
                    // here's a generic argument
                    for arg in genargs.args.iter() {
                        match arg {
                            syn::GenericArgument::Type(inner_ty) => {
                                res.append(&mut collect_all_lifetimes(inner_ty));
                            }
                            syn::GenericArgument::Lifetime(lifetime) => {
                                res.push(lifetime.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
            return res;
        }
        Type::Ptr(syn::TypePtr { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Reference(syn::TypeReference { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Slice(syn::TypeSlice { elem, .. }) => {
            return collect_all_lifetimes(elem);
        }
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            for elem in elems.iter() {
                res.append(&mut collect_all_lifetimes(elem))
            }
            return res;
        }
        _ => {
            return res;
        }
    }
}

/// Given a Generics object, return a new one that has the given type params added to it.
pub(crate) fn add_types(
    generics: &Generics,
    new_idents: impl IntoIterator<Item = Ident>,
) -> Generics {
    let mut generics = generics.clone();

    'outer: for ident in new_idents {
        // eprintln!("adding type {}", &ident);

        // avoid adding a duplicate
        for param in generics.params.iter() {
            if let syn::GenericParam::Type(type_param) = param {
                if type_param.ident == ident {
                    // do not add this type parameter a second time
                    continue 'outer;
                }
            }
        }

        generics.params.push(syn::GenericParam::Type(ident.into()));
    }

    generics
}

/// Given a Generics object, return a new one that has the given lifetime params added to it.
pub(crate) fn add_lifetimes(
    generics: &Generics,
    new_lifetimes: impl IntoIterator<Item = syn::Lifetime>,
) -> Generics {
    let mut generics = generics.clone();

    'outer: for lifetime in new_lifetimes {
        // eprintln!("adding type {}", &ident);

        // avoid adding a duplicate
        for param in generics.params.iter() {
            if let syn::GenericParam::Lifetime(existing_lifetime) = param {
                if existing_lifetime.lifetime.ident == lifetime.ident {
                    // do not add this type parameter a second time
                    continue 'outer;
                }
            }
        }

        generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeDef::new(lifetime)));
    }

    generics
}
