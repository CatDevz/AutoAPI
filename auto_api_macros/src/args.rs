use syn::parse::{Parse, ParseStream};

pub struct GenApiArguments {
    pub documentation: syn::LitStr,
}

impl Parse for GenApiArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(GenApiArguments {
            documentation: input.parse::<syn::LitStr>()?,
        })
    }
}
