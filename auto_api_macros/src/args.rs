use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, Ident, LitStr, Visibility,
};

pub struct GenApiArguments {
    pub documentation: LitStr,
}

impl Parse for GenApiArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(GenApiArguments {
            documentation: input.parse::<LitStr>()?,
        })
    }
}

pub struct GenApiModule {
    pub module_visibility: Visibility,
    _token: token::Mod,
    pub module_ident: Ident,
    _bracing: token::Brace,
    pub module_content: proc_macro2::TokenStream,
}

impl Parse for GenApiModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(GenApiModule {
            module_visibility: input.parse()?,
            _token: input.parse()?,
            module_ident: input.parse()?,
            _bracing: braced!(content in input),
            module_content: content.parse()?,
        })
    }
}
