use proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput, Type};

use darling::{ast, FromDeriveInput, FromField};

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
}

impl StructReceiver {
    fn try_to_tokens(&self) -> Result<TokenStream, syn::Error> {
        // get references to all our struct members so we can use them directly in quote!(...)
        let StructReceiver {
            ref ident,
            ref generics,
            ref data,
            ref from_type,
        } = *self;

        // handle generics
        let (imp, ty, wher) = generics.split_for_impl();

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        // format initializers for all fields
        let initializers = fields
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let span = field_ident.span();

                if let Some(true) = field.no_unwrap {
                    quote_spanned!(span=> #field_ident: other.#field_ident)
                } else {
                    quote_spanned!(span=> #field_ident: other.#field_ident.unwrap())
                }
            })
            .collect::<Vec<_>>();

        // code to check if unwrap will be successful
        let unwrap_checkers = fields
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let span = field_ident.span();

                if let Some(true) = field.no_unwrap {
                    quote!()
                } else {
                    quote_spanned! {span=>
                        if other.#field_ident.is_none() {
                            error.push(stringify!(#field_ident));
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

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

        Ok(quote! {
            impl #imp #ident #ty #wher {
                fn from_super_unwrap(other: #from_type) -> Self {
                    Self {
                        #(#initializers),*
                    }
                }

                fn from_super_try_unwrap(other: #from_type) -> ::std::result::Result<Self,#error_type> {
                    let mut error = #error_type::new();

                    #(#unwrap_checkers)*

                    if (error.any_missing()) {
                        return Err(error)
                    }

                    Ok( Self::from_super_unwrap(other) )
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

        })
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
    no_unwrap: Option<bool>,
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
