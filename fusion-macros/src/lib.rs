use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, ItemFn, Lit, Token};

struct MacroArgs {
    cache_name: String,
    ttl: Option<u64>,
    key_params: Vec<String>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cache_name = None;
        let mut ttl = None;
        let mut key_params = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        cache_name = Some(s.value());
                    } else {
                        return Err(syn::Error::new(lit.span(), "name must be a string literal"));
                    }
                }
                "ttl" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Int(i) = lit {
                        ttl = Some(i.base10_parse()?);
                    } else {
                        return Err(syn::Error::new(lit.span(), "ttl must be an integer"));
                    }
                }
                "key" => {
                    let expr: Expr = input.parse()?;
                    if let Expr::Path(ref path) = expr {
                        if let Some(ident) = path.path.get_ident() {
                            key_params.push(ident.to_string());
                        } else {
                            return Err(syn::Error::new_spanned(
                                expr,
                                "key must be a parameter name",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            expr,
                            "key must be a parameter name",
                        ));
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown attribute `{}`, expected `name`, `ttl`, or `key`",
                            other
                        ),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let cache_name = cache_name
            .ok_or_else(|| syn::Error::new(input.span(), "missing required attribute `name`"))?;

        Ok(MacroArgs {
            cache_name,
            ttl,
            key_params,
        })
    }
}

#[proc_macro_attribute]
pub fn app_cached(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let cache_name = args.cache_name;
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let param_names: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.clone());
                }
            }
            None
        })
        .collect();

    let key_idents: Vec<_> = if args.key_params.is_empty() {
        param_names.clone()
    } else {
        args.key_params
            .iter()
            .filter_map(|name| param_names.iter().find(|p| p.to_string() == *name).cloned())
            .collect()
    };

    let cache_key_expr = if key_idents.len() == 1 {
        let key = &key_idents[0];
        quote! { format!("{}:{}", #cache_name, #key) }
    } else {
        quote! { format!("{}:{}", #cache_name, format!("{:?}", (#(#key_idents),*))) }
    };

    let ttl_expr = if let Some(ttl_val) = args.ttl {
        quote! { Some(#ttl_val) }
    } else {
        quote! { None }
    };

    let expanded = quote! {
        #fn_vis async fn #fn_name(#fn_inputs) #fn_output {
            let cache_key = #cache_key_expr;

            if let Some(cache) = crate::cache::get_cache() {
                if let Ok(Some(bytes)) = cache.get(&cache_key).await {
                    if let Ok(value) = serde_json::from_slice(&bytes) {
                        return Ok(value);
                    }
                }
            }

            let result = (|| async #fn_body)().await;

            if let Ok(ref value) = result {
                if let Some(cache) = crate::cache::get_cache() {
                    if let Ok(bytes) = serde_json::to_vec(value) {
                        let _ = cache.set(&cache_key, bytes, #ttl_expr).await;
                    }
                }
            }

            result
        }
    };

    TokenStream::from(expanded)
}
