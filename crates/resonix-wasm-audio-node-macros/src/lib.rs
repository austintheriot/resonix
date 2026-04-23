use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, ItemStruct, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct ExportArgs {
    init: Expr,
}

impl Parse for ExportArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "init" {
            return Err(syn::Error::new(ident.span(), "expected `init`"));
        }

        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;

        Ok(ExportArgs { init: expr })
    }
}

// TODO: allow not passing in an `init` function, just allow a ZST if desired
#[proc_macro_attribute]
pub fn wasm_audio_node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ExportArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let init_expr = args.init;

    let expanded = quote! {
        #input

        #[unsafe(no_mangle)]
        pub extern "C" fn init() {
            resonix_wasm_audio_node::register_audio_node(#init_expr);
        }

        resonix_wasm_audio_node::export_wasm_api!();
    };

    TokenStream::from(expanded)
}
