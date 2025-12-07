//! Implementation of the `ConstBuilder` derive macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, Generics, Ident, Type, Visibility, parse2};

/// Entry point for the derive macro.
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = match parse2(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error(),
    };

    match generate(&input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

/// Generates the builder type and associated impls.
fn generate(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_data = match &input.data {
        Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ConstBuilder can only be derived for structs",
            ));
        }
    };

    let fields = match &struct_data.fields {
        Fields::Named(named) => &named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ConstBuilder requires a struct with named fields",
            ));
        }
    };

    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);
    let vis = &input.vis;
    let generics = &input.generics;

    // Collect field information
    let field_info: Vec<FieldInfo> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap().clone();
            let ty = f.ty.clone();
            let is_option = is_option_type(&ty);
            FieldInfo {
                name,
                ty,
                is_option,
            }
        })
        .collect();

    let builder_struct = generate_builder_struct(vis, &builder_name, generics, &field_info);
    let builder_impl = generate_builder_impl(&builder_name, struct_name, generics, &field_info);
    let entity_builder_method =
        generate_entity_builder_method(struct_name, &builder_name, generics);

    Ok(quote! {
        #builder_struct
        #builder_impl
        #entity_builder_method
    })
}

struct FieldInfo {
    name: Ident,
    ty: Type,
    is_option: bool,
}

/// Check if a type is `Option<T>`.
fn is_option_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    if let Some(segment) = type_path.path.segments.last() {
        return segment.ident == "Option";
    }
    false
}

/// Generate the builder struct definition.
fn generate_builder_struct(
    vis: &Visibility,
    builder_name: &Ident,
    generics: &Generics,
    fields: &[FieldInfo],
) -> TokenStream {
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    let builder_fields = fields.iter().map(|f| {
        let name = &f.name;
        let ty = &f.ty;
        quote! { #name: Option<#ty> }
    });

    // Generate PhantomData fields for unused lifetime parameters
    let phantom_fields = generics.params.iter().filter_map(|p| {
        if let GenericParam::Lifetime(lt) = p {
            let lt = &lt.lifetime;
            let phantom_name = format_ident!("_phantom_{}", lt.ident);
            Some(quote! { #phantom_name: core::marker::PhantomData<&#lt ()> })
        } else {
            None
        }
    });

    quote! {
        #[derive(Clone)]
        #vis struct #builder_name #impl_generics #where_clause {
            #(#builder_fields,)*
            #(#phantom_fields,)*
        }
    }
}

/// Generate the builder impl with `new()`, setters, and `build()`.
fn generate_builder_impl(
    builder_name: &Ident,
    struct_name: &Ident,
    generics: &Generics,
    fields: &[FieldInfo],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // new() initializes all fields to None
    let new_fields = fields.iter().map(|f| {
        let name = &f.name;
        quote! { #name: None }
    });

    let phantom_inits = generics.params.iter().filter_map(|p| {
        if let GenericParam::Lifetime(lt) = p {
            let phantom_name = format_ident!("_phantom_{}", lt.lifetime.ident);
            Some(quote! { #phantom_name: core::marker::PhantomData })
        } else {
            None
        }
    });

    // Setter methods
    let setters = fields.iter().map(|f| {
        let name = &f.name;
        let ty = &f.ty;
        quote! {
            #[must_use]
            pub const fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    // Build method: for Option fields, default to None; for required fields, panic if missing
    let build_fields = fields.iter().map(|f| {
        let name = &f.name;
        let name_str = name.to_string();
        if f.is_option {
            // Optional field: if builder field is Some(Some(x)) -> Some(x), Some(None) -> None, None -> None
            // Actually we store Option<Option<T>>, where outer is "was set", inner is the actual value.
            // But we simplified: builder stores Option<OriginalType>. If original is Option<T>, builder stores Option<Option<T>>.
            // For optional fields: unwrap_or(None) works.
            quote! {
                #name: match self.#name {
                    Some(v) => v,
                    None => None,
                }
            }
        } else {
            // Required field: panic if not set
            quote! {
                #name: match self.#name {
                    Some(v) => v,
                    None => panic!(concat!("missing required field: ", #name_str)),
                }
            }
        }
    });

    quote! {
        impl #impl_generics #builder_name #ty_generics #where_clause {
            /// Create a new builder with all fields unset.
            #[must_use]
            pub const fn new() -> Self {
                Self {
                    #(#new_fields,)*
                    #(#phantom_inits,)*
                }
            }

            #(#setters)*

            /// Build the final struct. Panics (at compile time in const context) if required fields are missing.
            #[must_use]
            pub const fn build(self) -> #struct_name #ty_generics {
                #struct_name {
                    #(#build_fields,)*
                }
            }
        }
    }
}

/// Generate the `::builder()` method on the original struct.
fn generate_entity_builder_method(
    struct_name: &Ident,
    builder_name: &Ident,
    generics: &Generics,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            /// Create a new builder for this type.
            #[must_use]
            pub const fn builder() -> #builder_name #ty_generics {
                #builder_name::new()
            }
        }
    }
}
