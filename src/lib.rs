// --------------------------------------------------
// external
// --------------------------------------------------
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
use quote::{
    quote,
    ToTokens,
};
use proc_macro::TokenStream;

#[proc_macro_derive(EnumConst, attributes(value, armtype))]
/// Add's constants to each arm of an enum
/// 
/// To get the value, call the function [`<enum_name>::value`]
/// 
/// # Example
/// 
/// ```
/// use enum_const::EnumConst;
/// 
/// #[derive(EnumConst)]
/// #[armtype("i32")]
/// enum MyEnum {
///     #[value = 0]
///     A,
///     #[value = 1]
///     B,
/// }
/// 
/// #[derive(EnumConst)]
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
///     assert_eq!(MyEnum::A.value(), 0);
///     assert_eq!(MyEnum::B.value(), 1);
///     assert_eq!(Tags::Key.value(), b"\x00\x01\x7f");
///     assert_eq!(Tags::Length.value(), b"\xba\x5e");
///     assert_eq!(Tags::Data.value(), b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f");
/// }
/// ```
pub fn enum_const(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // --------------------------------------------------
    // extract the name, variants, and values
    // --------------------------------------------------
    let enum_name = &input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("`EnumConst` can only be derived for enums"),
    };
    // --------------------------------------------------
    // extract the type
    // --------------------------------------------------
    let (type_name, deref) = match get_type(&input.attrs) {
        (Some(type_name), deref) => (type_name, deref),
        (None, _) => panic!("Missing #[armtype = ...] attribute, expected for `EnumConst`"),
    };
    // --------------------------------------------------
    // generate the output tokens and return
    // --------------------------------------------------
    let variant_code = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let value = match get_val(&variant.attrs) {
                Ok(value) => value,
                Err(e) => panic!("{}", e),
            };
            match deref {
                true => quote! { #enum_name::#variant_name => #value, },
                false => quote! { #enum_name::#variant_name => &#value, },
            }
        }
    );
    let expanded = quote! {
        impl #enum_name {
            #[inline]
            /// Returns the value of the enum variant
            /// defined by [`EnumConst`]
            /// 
            /// # Return
            /// 
            #[doc = concat!("* [`&'static ", stringify!(#type_name), "`]")]
            pub fn value(&self) -> &'static #type_name {
                match self {
                    #( #variant_code )*
                }
            }
        }
        impl ::std::cmp::PartialEq<#type_name> for #enum_name {
            #[inline]
            fn eq(&self, other: &#type_name) -> bool {
                self.value() == other
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumConstAny, attributes(value, armtype))]
/// Add's constants of any type to each arm of an enum
/// 
/// To get the value, the type must be explicitly passed
/// as a generic to [`<enum_name>::value`]. Upon failure,
/// it will return [`None`].
/// 
/// # Example
/// 
/// ```
/// use enum_const::EnumConst;
/// 
/// #[derive(EnumConst)]
/// #[armtype("i32")]
/// enum MyEnum {
///     #[value = 0]
///     A,
///     #[value = 1]
///     B,
/// }
/// 
/// #[derive(EnumConst)]
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
///     assert_eq!(MyEnum::A.value(), 0);
///     assert_eq!(MyEnum::B.value(), 1);
///     assert_eq!(Tags::Key.value(), b"\x00\x01\x7f");
///     assert_eq!(Tags::Length.value(), b"\xba\x5e");
///     assert_eq!(Tags::Data.value(), b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f");
/// }
/// ```
pub fn enum_const_any(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // --------------------------------------------------
    // extract the name, variants, and values
    // --------------------------------------------------
    let enum_name = &input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("`EnumConstAny` can only be derived for enums"),
    };
    // --------------------------------------------------
    // generate the output tokens and return
    // --------------------------------------------------
    let variant_code = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_type = get_type(&variant.attrs);
        if variant_type.0.is_none() {
            panic!("Missing #[armtype = ...] attribute, expected for `EnumConstAny`");
        }
        let variant_type = variant_type.0.unwrap();
        match get_val(&variant.attrs) {
            Ok(value) => quote! {
                #enum_name::#variant_name => {
                    // if ::std::any::TypeId::of::<T>() != ::std::any::TypeId::of::<#variant_type>() {
                    let val: dyn ::std::any::Any = #value;
                    return val.downcast_ref::<&'static T>();
                    // }
                    // Some(&#value as &'static T)
                    // Some(&#value as &T)
                },
            },
            Err(_) => quote! { #enum_name::#variant_name => None, },
        }
    });
    let expanded = quote! {
        impl #enum_name {
            // pub fn value<T: 'static>(&self) -> Option<&(dyn ::std::any::Any + 'static)> {
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
fn get_val(attrs: &[Attribute]) -> Result<proc_macro2::TokenStream, syn::Error> {
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
                return Err(syn::Error::new_spanned("", "Attemping to parse non-literal attribute for `value`: not yet supported"))
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
    Err(syn::Error::new_spanned("", "Missing #[value = ...] attribute, expected for `EnumConst` trait"))
}

/// Helper function to extract the type from the [`Attribute`], aka `#[armtype(<type>)]`
///
/// # Input
///
/// ```text
/// #[armtype(<type>)]
/// ```
///
/// # Output
///
/// * 0 - [`Type`] containing the type `<type>`, or [`None`] if the attribute is not present
/// * 1 - There is an additional flag that indicates if the type should be a dereferenced or not
fn get_type(attrs: &[Attribute]) -> (Option<Type>, bool) {
    for attr in attrs {
        if !attr.path.is_ident("armtype") { continue; }
        let tokens = match attr.parse_args::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens,
            Err(_) => return (None, false),
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
        return (syn::parse2::<Type>(tokens).ok(), deref);
    }
    (None, false)
}