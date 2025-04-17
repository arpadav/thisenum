use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use crate::Ctxt;
use crate::ast::MainContainer;

pub fn derive(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // --------------------------------------------------
    // create a new error context
    // --------------------------------------------------
    let cx = Ctxt::new();
    // --------------------------------------------------
    // get the parsed container for Const derive
    // --------------------------------------------------
    let cont = match MainContainer::from_ast(&cx, input) {
        Some(cont) => cont,
        None => return Err(cx.check().unwrap_err()),
    };
    // --------------------------------------------------
    // check for errors
    // --------------------------------------------------
    cx.check()?;

    match &cont.data[0].attrs.value {
        Some(x) => {
            panic!("{}", x.to_token_stream());
        }
        None => panic!("BRUH")
    }
    
    // --------------------------------------------------
    // < crate level validation >
    // --------------------------------------------------

    // --------------------------------------------------
    // init
    // --------------------------------------------------
    let mut expanded = quote! {};

    // --------------------------------------------------
    // < impls >
    // --------------------------------------------------
    // if let (
    //     Some(key_enc),
    //     Some(len_enc),
    //     true,
    // ) = (
    //     cont.attrs.key.enc.as_ref(),
    //     cont.attrs.len.enc.as_ref(),
    //     all_encoders_exist,
    // ) {
    //     let encode_impls = encode_impl::gen_encode_impl(
    //         &cont,
    //         &key_enc,
    //         &len_enc,
    //     );
    //     expanded = quote! {
    //         #expanded
    //         #encode_impls
    //     }
    // }

    Ok(expanded)
}