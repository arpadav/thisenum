// --------------------------------------------------
// external
// --------------------------------------------------
use quote::ToTokens;

// All symbols (all idents, e.g. no kebab-case)
pub(crate) const VALUE: Symbol = Symbol("value");
pub(crate) const ARMTYPE: Symbol = Symbol("armtype");

/// Container symbols
pub(crate) static CONT_SYMBOLS: Symbols = Symbols(&[
    ARMTYPE
]);

/// Variant symbols
pub(crate) static VARIANT_SYMBOLS: Symbols = Symbols(&[
    VALUE,
]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A symbol for `thisenum` attributes
pub(crate) struct Symbol(pub(crate) &'static str);

/// [`Symbol`] implementation of [`std::fmt::Display`]
impl std::fmt::Display for Symbol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}
/// [`Symbol`] implementation of [`From`] for [`syn::Path`]
impl From<&syn::Path> for Symbol {
    fn from(path: &syn::Path) -> Self {
        let ident = path.segments
            .last()
            .expect("path has no segments")
            .ident
            .to_string();
        Symbol(Box::leak(ident.into_boxed_str()))
    }
}
/// [`syn::Path`] implementation of [`From`] for [`Symbol`]
impl From<Symbol> for syn::Path {
    fn from(symbol: Symbol) -> Self {
        syn::parse_str(symbol.0).expect("?")
    }
}
/// [`Symbol`] implementation of [`ToTokens`]
impl ToTokens for Symbol {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        #[allow(clippy::unwrap_used)] // all symbols are idents
        let ident: syn::Ident = syn::parse_str(self.0).unwrap();
        tokens.extend(quote::quote! { #ident })
    }
}
/// [`syn::Path`] implementation of [`PartialEq`] for [`Symbol`]
impl PartialEq<Symbol> for syn::Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}
/// [`&syn::Path`](syn::Path) implementation of [`PartialEq`] for [`Symbol`]
impl PartialEq<Symbol> for &syn::Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

/// Multiple symbols, for displaying errors
pub(crate) struct Symbols<'a>(&'a [Symbol]);

/// [`Symbols`] implementation of [`std::fmt::Display`]
impl std::fmt::Display for Symbols<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut symbols = self.0.to_vec();
        // reverse alphabetical
        symbols.sort_by(|a, b| b.0.cmp(a.0));
        symbols.iter().try_for_each(|symbol| write!(f, "`{}` ", symbol.0))
    }
}
