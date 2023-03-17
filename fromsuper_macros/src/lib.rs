use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput, Type};

use darling::{ast, FromDeriveInput, FromField};

mod generics;

/// The struct that contains all the info about the to-be-derived struct.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(from_super), supports(struct_named))]
struct StructReceiver {
    /// The struct ident
    ident: syn::Ident,

    /// The type's generics
    generics: syn::Generics,

    /// The body of the struct. The first type parameter is `()` because we only
    /// accept proper structs, no enums or tuple structs.
    data: ast::Data<(), FieldReceiver>,

    /// Option to specify the original (super) type to convert our derived type from.
    from_type: Type,

    /// Option to specify whether to unpack the single struct members
    unpack: Option<bool>,
}

impl StructReceiver {
    fn try_to_tokens(&self) -> Result<TokenStream, syn::Error> {
        // get references to all our struct members so we can use them directly in quote!(...)
        let StructReceiver {
            ref ident,
            ref generics,
            ref data,
            ref from_type,
            ref unpack,
        } = *self;

        // whether to unpack any member
        let unpack = unpack.unwrap_or(false);

        // handle generics
        let (_, ty, wher) = generics.split_for_impl();

        // adapt generics of impl block to include type parameters used in the
        // super struct but not in the sub struct
        let extra_super_tyidents = generics::merge_generics(from_type, generics)?;
        let new_generics = generics::add_types(generics, extra_super_tyidents);
        let (imp, _, _) = new_generics.split_for_impl();

        // eprintln!("ident: {:?}", ident);
        // eprintln!("generics: {:?}", generics);
        // eprintln!("from_type: {:?}", from_type);
        // eprintln!("imp: {:?}", imp);
        // eprintln!("ty: {:?}", ty);
        // eprintln!("wher: {:?}", wher);
        // eprintln!("");

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        return Ok(if unpack {
            // Implement TryFrom

            let error_type = format_ident!(
                "{}FromSuperError_{}",
                ident,
                from_type
                    .to_token_stream()
                    .to_string()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            );

            // code to check if unwrap will be successful
            let unwrap_checkers = fields
                .iter()
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    let span = field_ident.span();
                    let source_ident = field.rename_from.as_ref().unwrap_or(field_ident);

                    if let Some(true) = field.no_unpack {
                        quote!()
                    } else {
                        quote_spanned! {span=>
                            if value.#source_ident.is_none() {
                                error.push(stringify!(#field_ident));
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let initializers = fields
                .iter()
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    let span = field_ident.span();
                    let source_ident = field.rename_from.as_ref().unwrap_or(field_ident);

                    if let Some(true) = field.no_unpack {
                        quote_spanned!(span=> #field_ident: value.#source_ident)
                    } else {
                        quote_spanned!(span=> #field_ident: value.#source_ident.unwrap())
                    }
                })
                .collect::<Vec<_>>();

            quote!(
                impl #imp ::std::convert::TryFrom<#from_type> for #ident #ty #wher {
                    type Error = #error_type;

                    fn try_from(value: #from_type) -> ::std::result::Result<Self, Self::Error> {
                        let mut error = #error_type::new();

                        #(#unwrap_checkers)*

                        if (error.any_missing()) {
                            return Err(error)
                        }

                        Ok( Self {
                            #(#initializers),*
                        } )
                    }
                }

                #[allow(non_camel_case_types)]
                #[derive(PartialEq, Debug)]
                struct #error_type {
                    missing: Vec<&'static str>,
                }

                impl #error_type {
                    fn new() -> Self { Self { missing: Vec::new() }}

                    fn push(&mut self, missing: &'static str) {
                        self.missing.push(missing);
                    }

                    fn any_missing(&self) -> bool {
                        self.missing.len() > 0
                    }
                }

                impl ::std::fmt::Display for #error_type {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(f, "Attribute(s) ")?;

                        for (i, missing) in self.missing.iter().enumerate() {
                            write!(f, "{}", missing)?;
                            if i+1 < self.missing.len() {
                                write!(f, ", ")?;
                            }
                        }

                        write!(f, " of the super struct {} not initialized", stringify!(#from_type))?;
                        Ok(())
                    }
                }

                impl ::std::error::Error for #error_type { }
            )
        } else {
            // Implement From

            let initializers = fields
                .iter()
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    let span = field_ident.span();
                    let source_ident = field.rename_from.as_ref().unwrap_or(field_ident);

                    quote_spanned!(span=> #field_ident: value.#source_ident)
                })
                .collect::<Vec<_>>();

            quote!(
                impl #imp ::std::convert::From<#from_type> for #ident #ty #wher {
                    fn from(value: #from_type) -> Self {
                        Self {
                            #(#initializers),*
                        }
                    }
                }
            )
        });
    }
}

/// The handler for each field within the provided struct
#[derive(Debug, FromField)]
#[darling(attributes(from_super))]
struct FieldReceiver {
    /// Get the ident of the field. This is an Option to accommodate tuples or
    /// tuple structs (`None` in this case). However, this cannot happen in our
    /// case as we only allow normal structs.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    #[allow(dead_code)]
    ty: syn::Type,

    /// Option to not unwrap or unpack this field.
    no_unpack: Option<bool>,

    /// Option to take this field's value from a differently-named source field
    rename_from: Option<syn::Ident>,
}

#[proc_macro_derive(FromSuper, attributes(from_super))]
pub fn derive_fromsuper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // load the struct's raw AST
    let parsed_raw = parse_macro_input!(input as DeriveInput);

    // Parse it into our custom type using darling
    let struct_receiver = StructReceiver::from_derive_input(&parsed_raw).unwrap();

    struct_receiver
        .try_to_tokens()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
