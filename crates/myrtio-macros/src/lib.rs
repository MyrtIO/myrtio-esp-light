extern crate proc_macro;

use proc_macro::TokenStream;

/// Derive macro that generates a const-capable builder for structs.
///
/// Usage:
/// ```ignore
/// #[derive(ConstBuilder)]
/// pub struct MyEntity<'a> {
///     pub name: &'a str,
///     pub count: u32,
///     pub optional: Option<u8>,
/// }
/// ```
///
/// This generates:
/// - `MyEntityBuilder<'a>` – a builder type with all fields wrapped in `Option`.
/// - `MyEntityBuilder::new()` – const constructor returning builder with all `None`.
/// - `MyEntityBuilder::name(self, val) -> Self`, etc. – const setters for each field.
/// - `MyEntityBuilder::build(self) -> MyEntity` – const finalizer; panics at compile time
///   if required (non-`Option`) fields are missing.
/// - `MyEntity::builder() -> MyEntityBuilder` – convenience const fn.
#[proc_macro_derive(ConstBuilder, attributes(const_builder))]
pub fn derive_const_builder(input: TokenStream) -> TokenStream {
    const_builder::derive(input.into()).into()
}

mod const_builder;
