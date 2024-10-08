#![doc = include_str!("../README.md")]
// --------------------------------------------------
// external
// --------------------------------------------------
use quote::{
    quote,
    ToTokens,
};
use syn::{
    Meta,
    Data,
    Type,
    DataEnum,
    Attribute,
    DeriveInput,
    MetaNameValue,
    parse_macro_input,
};
use unzip_n::unzip_n;
use thiserror::Error;
use proc_macro::TokenStream;

// --------------------------------------------------
// local
// --------------------------------------------------
mod prelude;
use prelude::*;
unzip_n!(3);

#[derive(Error, Debug)]
/// All errors that can occur while deriving [`Const`]
/// or [`ConstEach`]
enum Error {
    #[error("`{0}` can only be derived for enums")]
    DeriveForNonEnum(String),
    #[error("Missing #[armtype = ...] attribute {0}, required for `{1}`-derived enum")]
    MissingArmType(String, String),
    #[error("Missing #[value = ...] attribute, expected for `{0}`-derived enum")]
    MissingValue(String),
    #[error("Attemping to parse non-literal attribute for `value`: not yet supported")]
    NonLiteralValue,
}

#[proc_macro_derive(Const, attributes(value, armtype))]
/// Add's constants to each arm of an enum
/// 
/// * To get the value as a reference, call the function [`<enum_name>::value`]
/// * However, direct comparison to non-reference values are possible with
///   [`PartialEq`]
/// 
/// The `#[armtype = ...]` attribute is required for this macro to function, 
/// and must be applied to **the enum**, since all values share the same type.
/// 
/// All values set will return a [`&'static T`] reference. To the input type,
/// of [`T`] AND [`&T`]. If multiple references are used (e.g. `&&T`), then
/// the return type will be [`&'static &T`].
/// 
/// # Example
/// 
/// ```
/// use thisenum::Const;
/// 
/// #[derive(Const, Debug)]
/// #[armtype(i32)]
/// enum MyEnum {
///     #[value = 0]
///     A,
///     #[value = 1]
///     B,
/// }
/// 
/// #[derive(Const, Debug)]
/// #[armtype(&[u8])]
/// enum Tags {
///     #[value = b"\x00\x01\x7f"]
///     Key,
///     #[value = b"\xba\x5e"]
///     Length,
///     #[value = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f"]
///     Data,
/// }
/// 
/// fn main() {
///     // it's prefered to use the function call to `value` 
///     // to get a [`&'static T`] reference to the value
///     assert_eq!(MyEnum::A.value(), &0);
///     assert_eq!(MyEnum::B.value(), &1);
///     assert_eq!(Tags::Key.value(), b"\x00\x01\x7f");
///     assert_eq!(Tags::Length.value(), b"\xba\x5e");
///     assert_eq!(Tags::Data.value(), b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f");
/// 
///     // can also check equality without the function call. This must compare the input 
///     // type defined in `#[armtype = ...]`
///     //
///     // to use this, use the `eq` feature in `Cargo.toml`: thisenum = { version = "x", features = ["eq"] }
///     #[cfg(feature = "eq")]
///     assert_eq!(Tags::Length, b"\xba\x5e");
/// }
/// ```
pub fn thisenum_const(input: TokenStream) -> TokenStream {
    let name = "Const";
    let input = parse_macro_input!(input as DeriveInput);
    // --------------------------------------------------
    // extract the name, variants, and values
    // --------------------------------------------------
    let enum_name = &input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("{}", Error::DeriveForNonEnum(name.into())),
    };
    // --------------------------------------------------
    // extract the type
    // --------------------------------------------------
    let (type_name, deref) = match get_deref_type(&input.attrs) {
        Some((type_name, deref)) => (type_name, deref),
        None => panic!("{}", Error::MissingArmType("applied to enum".into(), name.into())),
    };
    let type_name_raw = match get_type(&input.attrs) {
        Some(type_name_raw) => type_name_raw,
        None => panic!("{}", Error::MissingArmType("applied to enum".into(), name.into())),
    };
    // --------------------------------------------------
    // get unique assigned values
    // --------------------------------------------------
    let values = variants
        .iter()
        .map(|variant| get_val(name.into(), &variant.attrs))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let values_string = values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let repeated_values_string = values_string.clone().into_iter().repeated();
    // --------------------------------------------------
    // generate the output tokens
    // --------------------------------------------------
    let (
        debug_arms,
        variant_match_arms,
        mut variant_inv_match_arms
    ) = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            // ------------------------------------------------
            // number of args in the variant
            // ------------------------------------------------
            // e.g.: enum Test { VariantA(i23), VariantB(String, String) }
            // will have 1 (i23) and 2 (String, String)
            // ------------------------------------------------
            let num_args = match variant.fields {
                syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => named.len(),
                syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => unnamed.len(),
                syn::Fields::Unit => 0,
            };
            let value = match get_val(name.into(), &variant.attrs) {
                Ok(value) => value,
                Err(e) => panic!("{}", e),
            };
            // ------------------------------------------------
            // check if the value is unique
            // this is used to prevent unreachable arms
            // ------------------------------------------------
            let val_repeated = repeated_values_string.contains(&value.to_string());
            // ------------------------------------------------
            // if the type input is a reference (e.g. &[u8] or &str)
            // then the return type will be 
            // * `&'static [u8]` or
            // * `&'static str`
            //
            // otherwise, if the input is not a reference (e.g. u8 or f32)
            // then the return type will be
            // * `&'static u8` or
            // * `&'static f32`
            //
            // as a result, need to ensure we are removing / adding
            // the `&` symbol wherever necessary
            // ------------------------------------------------
            let args_tokens = match num_args {
                0 => quote! {},
                _ => {
                    let args = (0..num_args).map(|_| quote! { _ });
                    quote! { ( #(#args),* ) }
                },
            };
            // ------------------------------------------------
            // debug arms implementation
            // ------------------------------------------------
            let debug_arm = match get_val(name.into(), &variant.attrs) {
                Ok(_) => quote! { #enum_name::#variant_name #args_tokens => write!(f, concat!(stringify!(#enum_name), "::", stringify!(#variant_name), ": {:?}"), self.value()), },
                Err(e) => panic!("{}", e),
            };
            // ------------------------------------------------
            // variant -> value
            // ------------------------------------------------
            let vma = match deref {
                true => quote! { #enum_name::#variant_name #args_tokens => #value, },
                false => quote! { #enum_name::#variant_name #args_tokens => &#value, },
            };
            // ------------------------------------------------
            // value -> variant
            // ------------------------------------------------
            match (num_args, val_repeated) {
                (0, false) => (debug_arm, vma, Some(quote! { #value => Ok(#enum_name::#variant_name), })),
                (_, _) => (debug_arm, vma, None),
            }
        })
        .into_iter()
        .unzip_n_vec();
    // --------------------------------------------------
    // get the vima for repeated values
    // --------------------------------------------------
    let mut repeated_indices = values_string
        .clone()
        .into_iter()
        .repeated_idx();
    repeated_indices.sort_by(|a, b| b.cmp(a));
    repeated_indices
        .iter()
        .for_each(|i| { variant_inv_match_arms.remove(*i); } );
    let variant_inv_match_arms_repeated = values_string
        .clone()
        .into_iter()
        .positions()
        .iter()
        .map(|(_, pos)| match pos.len() {
            ..=1 => quote! {},
            _ => {
                let val = values[pos[0]].clone();
                quote! { #val => Err(::thisenum::Error::UnreachableValue(format!("{:?}", #val))), }
            }
        })
        .collect::<Vec<_>>();
    // --------------------------------------------------
    // get all the indices of variants which have nested args
    // --------------------------------------------------
    let arg_indices = variant_inv_match_arms
        .iter()
        .enumerate()
        .filter(|(i, v)| v.is_none() && !repeated_indices.contains(&i))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let variant_inv_match_arms_args = values
        .clone()
        .into_iter()
        .zip(variants)
        .enumerate()
        .filter(|(i, _)| arg_indices.contains(i))
        .map(|(_, (value, variant))| {
            let variant_name = &variant.ident;
            quote! { #value => Err(::thisenum::Error::UnableToReturnVariant(stringify!(#variant_name).into())), }
        })
        .collect::<Vec<_>>();
    // --------------------------------------------------
    // see deref comment above
    // --------------------------------------------------
    let variant_par_eq_lhs = match deref {
        true => quote! { &self.value() == other },
        false => quote! { self.value() == other },
    };
    let variant_par_eq_rhs = match deref {
        true => quote! { &other.value() == self },
        false => quote! { other.value() == self },
    };
    let into_impl = match deref {
        false => quote! {
            #[automatically_derived]
            #[doc = concat!(" [`Into`] implementation for [`", stringify!(#enum_name), "`]")]
            impl ::std::convert::Into<#type_name_raw> for #enum_name {
                #[inline]
                fn into(self) -> #type_name_raw {
                    *self.value()
                }
            }
        },
        true => quote! { },
    };
    // --------------------------------------------------
    // return
    // --------------------------------------------------
    let mut expanded = quote! {
        #[automatically_derived]
        impl #enum_name {
            #[inline]
            /// Returns the value of the enum variant
            /// defined by [`Const`]
            /// 
            /// # Returns
            /// 
            #[doc = concat!(" * [`&'static ", stringify!(#type_name), "`]")]
            pub fn value(&self) -> &'static #type_name {
                match self {
                    #( #variant_match_arms )*
                }
            }
        }
        #[automatically_derived]
        #[cfg(feature = "eq")]
        #[doc = concat!(" [`PartialEq<", stringify!(#type_name_raw) ,">`] implementation for [`", stringify!(#enum_name), "`]")]
        ///
        #[doc = concat!(" This is the LHS of the [`PartialEq`] implementation between [`", stringify!(#enum_name), "`] and [`", stringify!(#type_name_raw), "`]")]
        /// 
        /// # Returns
        /// 
        /// * [`true`] if the type and the enum are equal
        /// * [`false`] if the type and the enum are not equal
        impl ::std::cmp::PartialEq<#type_name_raw> for #enum_name {
            #[inline]
            fn eq(&self, other: &#type_name_raw) -> bool {
                #variant_par_eq_lhs
            }
        }
        #[automatically_derived]
        #[cfg(feature = "eq")]
        #[doc = concat!(" [`PartialEq<", stringify!(#enum_name) ,">`] implementation for [`", stringify!(#type_name_raw), "`]")]
        /// 
        #[doc = concat!(" This is the RHS of the [`PartialEq`] implementation between [`", stringify!(#enum_name), "`] and [`", stringify!(#type_name_raw), "`]")]
        /// 
        /// # Returns
        /// 
        /// * [`true`] if the enum and the type are equal
        /// * [`false`] if the enum and the type are not equal
        impl ::std::cmp::PartialEq<#enum_name> for #type_name_raw {
            #[inline]
            fn eq(&self, other: &#enum_name) -> bool {
                #variant_par_eq_rhs
            }
        }
        #[automatically_derived]
        #[doc = concat!(" [`Debug`] implementation for [`", stringify!(#enum_name), "`]")]
        impl ::std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #( #debug_arms )*
                }
            }
        }
        #into_impl
    };
    let variant_inv_match_arms = variant_inv_match_arms.into_iter().filter(|v| v.is_some()).map(|v| v.unwrap());
    expanded = quote! {
        #expanded
        #[automatically_derived]
        #[doc = concat!(" [`TryFrom`] implementation for [`", stringify!(#enum_name), "`]")]
        ///
        /// This is able to be derived since none of the Arms of the Enum had
        /// any arguments. If that is the case, this implementation is 
        /// non-existent.
        /// 
        /// # Returns
        /// 
        /// * [`Ok(T)`] where `T` is the enum variant
        /// * [`Err(Error)`] if the conversion fails
        impl ::std::convert::TryFrom<#type_name_raw> for #enum_name {
            type Error = ::thisenum::Error;
            #[inline]
            fn try_from(value: #type_name_raw) -> Result<Self, Self::Error> {
                match value {
                    #( #variant_inv_match_arms )*
                    #( #variant_inv_match_arms_repeated )*
                    #( #variant_inv_match_arms_args )*
                    _ => Err(::thisenum::Error::InvalidValue(format!("{:?}", value), stringify!(#enum_name).into())),
                }
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(ConstEach, attributes(value, armtype))]
/// Add's constants of any type to each arm of an enum
/// 
/// To get the value, the type must be explicitly passed
/// as a generic to [`<enum_name>::value`]. This will automatically
/// try to convert constant to the expected type using [`std::any::Any`] 
/// and [`downcast_ref`]. Currently [`TryFrom`] is not supported, so typing
/// is fairly strict. Upon failure, it will return [`None`].
/// 
/// * To get the value as a reference, call the function [`<enum_name>::value`]
/// * Unlike [`Const`], this macro does not enable direct comparison
///   using [`PartialEq`] when imported using the `eq` feature.
/// 
/// The `#[armtype = ...]` attribute is **NOT*** required for this macro to function, 
/// but ***CAN** be applied to ***each individual arm*** of the enum, since values
/// are not expected to share a type. If no type is given, then the type is
/// inferred from the literal value in the `#[value = ...]` attribute.
/// 
/// All values set will return a [`Option<&'static T>`] reference. To the input type,
/// of [`T`] AND [`&T`]. If multiple references are used (e.g. `&&T`), then
/// the return type will be [`Option<&'static &T>`].
/// 
/// # Example
/// 
/// ```
/// use thisenum::ConstEach;
/// 
/// #[derive(ConstEach, Debug)]
/// enum MyEnum {
///     #[armtype(u8)]
///     #[value = 0xAA]
///     A,
///     #[value = "test3"]
///     B,
/// }
/// 
/// #[derive(ConstEach, Debug)]
/// enum Tags {
///     #[value = b"\x00\x01"]
///     Key,
///     #[armtype(u16)]
///     #[value = 24250]
///     Length,
///     #[armtype(&[u8])]
///     #[value = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f"]
///     Data,
/// }
/// 
/// fn main() {
///     // [`ConstEach`] examples
///     assert!(MyEnum::A.value::<u8>().is_some());
///     assert!(MyEnum::A.value::<Vec<f32>>().is_none());
///     assert!(MyEnum::B.value::<u8>().is_none());
///     assert!(MyEnum::B.value::<&str>().is_some());
///     assert!(Tags::Data.value::<&[u8]>().is_some());
/// 
///     // An infered type. This will be as strict as possible,
///     // therefore [`&[u8]`] will fail but [`&[u8; 2]`] will succeed
///     assert!(Tags::Key.value::<&[u8; 2]>().is_some());
///     assert!(Tags::Key.value::<&[u8; 5]>().is_none());
///     assert!(Tags::Key.value::<&[u8]>().is_none());
///     assert!(u16::from_le_bytes(**Tags::Key.value::<&[u8; 2]>().unwrap()) == 0x0100);
/// 
///     // casting as anything other than the defined / inferred type will
///     // fail, since this uses [`downcast_ref`] from [`std::any::Any`]
///     assert!(Tags::Length.value::<u16>().is_some());
///     assert!(Tags::Length.value::<u32>().is_none());
///     assert!(Tags::Length.value::<u64>().is_none());
/// 
///     // however, can always convert to a different type
///     // after value is successfully acquired
///     assert!(*Tags::Length.value::<u16>().unwrap() as u32 == 24250);
/// }
/// ```
pub fn thisenum_const_each(input: TokenStream) -> TokenStream {
    let name = "ConstEach";
    let input = parse_macro_input!(input as DeriveInput);
    // --------------------------------------------------
    // extract the name, variants, and values
    // --------------------------------------------------
    let enum_name = &input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("{}", Error::DeriveForNonEnum(name.into())),
    };
    // --------------------------------------------------
    // generate the output tokens
    // --------------------------------------------------
    let variant_code = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match (get_type(&variant.attrs), get_val(name.into(), &variant.attrs)) {
            // ------------------------------------------------
            // if type is specified, use it
            // ------------------------------------------------
            (Some(typ), Ok(value)) => quote! {
                #enum_name::#variant_name => {
                    let val: &dyn ::std::any::Any = &(#value as #typ);
                    val.downcast_ref::<T>()
                },

            },
            // ------------------------------------------------
            // no type specified, try to infer
            // ------------------------------------------------
            (None, Ok(value)) => quote! {
                #enum_name::#variant_name => {
                    let val: &dyn ::std::any::Any = &#value;
                    val.downcast_ref::<T>()
                },
            },
            // ------------------------------------------------
            // unable to infer type
            // ------------------------------------------------
            (_, Err(_)) => quote! { #enum_name::#variant_name => None, },
        }
    });
    // ------------------------------------------------
    // return
    // ------------------------------------------------
    let expanded = quote! {
        #[automatically_derived]
        #[doc = concat!(" [`ConstEach`] implementation for [`", stringify!(#enum_name), "`]")]
        impl #enum_name {
            pub fn value<T: 'static>(&self) -> Option<&'static T> {
                match self {
                    #( #variant_code )*
                    _ => None,
                }
            }
        }
    };
    TokenStream::from(expanded)
}

/// Helper function to extract the value from a [`MetaNameValue`], aka `#[value = <value>]`
///
/// # Input
///
/// ```text
/// #[value = <value>]
/// ```
///
/// # Output
///
/// [`TokenStream`] containing the value `<value>`, or [`Err`] if the attribute is not present / invalid
fn get_val(name: String, attrs: &[Attribute]) -> Result<proc_macro2::TokenStream, Error> {
    for attr in attrs {
        if !attr.path.is_ident("value") { continue; }
        match attr.parse_meta() {
            Ok(meta) => match meta {
                Meta::NameValue(MetaNameValue { lit, .. }) => return Ok(lit.into_token_stream()),
                Meta::List(list) => {
                    let tokens = list.nested.iter().map(|nested_meta| {
                        match nested_meta {
                            syn::NestedMeta::Lit(lit) => lit.to_token_stream(),
                            syn::NestedMeta::Meta(meta) => meta.to_token_stream(),
                        }
                    });
                    return Ok(quote! { #( #tokens )* });
                }
                Meta::Path(_) => return Ok(meta.into_token_stream())
            },
            Err(_) => {
                return Err(Error::NonLiteralValue);
                /*
                // Maybe for future:
                // --------------------------------------------------
                let elems = attr
                    .to_token_stream()
                    .to_string();
                // println!("elems: {}", elems);
                let mut elems = elems
                    .trim()
                    .trim_start_matches("#[")
                    .rsplit_once("]")
                    .unwrap()
                    .0
                    .split("=")
                    .collect::<Vec<_>>();
                // println!("elems: {:?}", elems);
                elems.remove(0);
                // println!("elems: {:?}", elems);
                return Ok(elems
                    .join("=")
                    .trim()
                    .parse::<proc_macro2::TokenStream>()?);
                // --------------------------------------------------
                */
            },
        }
    }
    Err(Error::MissingValue(name))
}

/// Helper function to extract the type from the [`Attribute`], aka `#[armtype(<type>)]`
/// 
/// Will indicate whether or not the type should be dereferenced or not. Useful
/// for the [`Const`] macro
///
/// # Input
///
/// ```text
/// #[armtype(<type>)]
/// ```
///
/// # Output
///
/// [`None`] if the attribute is not present / invalid
/// 
/// Otherwise a tuple:
/// 
/// * 0 - [`Type`] containing the type `<type>` (already de-referenced)
/// * 1 - An additional flag that indicates if the type has been de-referenced
fn get_deref_type(attrs: &[Attribute]) -> Option<(Type, bool)> {
    for attr in attrs {
        if !attr.path.is_ident("armtype") { continue; }
        let tokens = match attr.parse_args::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens,
            Err(_) => return None,
        };
        let deref = tokens
            .to_string()
            .trim()
            .starts_with('&');
        let tokens = match deref {
            true => {
                let mut tokens = tokens.into_iter();
                let _ = tokens.next();
                tokens.collect::<proc_macro2::TokenStream>()
            }
            false => tokens,
        };
        return match syn::parse2::<Type>(tokens).ok() {
            Some(type_name) => Some((type_name, deref)),
            None => None
        }
    }
    None
}

/// Helper function to extract the type from the [`Attribute`], aka `#[armtype(<type>)]`
/// 
/// Will return the raw [`Type`]. Useful for the [`Const`] and the [`ConstEach`]
/// macros
///
/// # Input
///
/// ```text
/// #[armtype(<type>)]
/// ```
///
/// # Output
///
/// [`None`] if the attribute is not present / invalid
/// 
/// Otherwise [`Some<Type>`] containing the type `<type>`
fn get_type(attrs: &[Attribute]) -> Option<Type> {
    for attr in attrs {
        if !attr.path.is_ident("armtype") { continue; }
        let tokens = match attr.parse_args::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens,
            Err(_) => return None,
        };
        return syn::parse2::<Type>(
            tokens
            .into_iter()
            .collect::<proc_macro2::TokenStream>()
        ).ok()
    }
    None
}