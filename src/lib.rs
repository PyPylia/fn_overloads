use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, TokenStreamExt};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Block, Generics, Ident, Result, ReturnType, Token, Type, Visibility, parse_quote,
};

mod kw {
    syn::custom_keyword!(Send);
}

#[allow(dead_code)]
enum Asyncness {
    Async {
        async_token: Token![async],
        not_sendness: Option<(Token![!], kw::Send)>,
    },
    Sync,
}

impl Parse for Asyncness {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(async_token) = input.parse()? {
            let not_sendness = if input.peek(Token![!]) && input.peek2(kw::Send) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            };

            Ok(Self::Async {
                async_token,
                not_sendness,
            })
        } else {
            Ok(Self::Sync)
        }
    }
}

#[allow(dead_code)]
struct OverloadArg {
    name: Ident,
    colon: Token![:],
    ty: Type,
}

impl Parse for OverloadArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        Ok(Self { name, colon, ty })
    }
}

#[allow(dead_code)]
struct Overload {
    asyncness: Asyncness,
    generics: Generics,
    paren: Paren,
    args: Punctuated<OverloadArg, Token![,]>,
    ret: ReturnType,
    block: Block,
}

impl Parse for Overload {
    fn parse(input: ParseStream) -> Result<Self> {
        let asyncness = input.parse()?;
        let generics = input.parse()?;

        let content;
        let paren = parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;

        let ret = input.parse()?;
        let block = input.parse()?;

        Ok(Self {
            asyncness,
            generics,
            paren,
            args,
            ret,
            block,
        })
    }
}

impl Overload {
    fn to_tokens(&self, name: &Ident) -> TokenStream2 {
        #[cfg(feature = "std")]
        let core_path = quote! { ::std };

        #[cfg(not(feature = "std"))]
        let core_path = quote! { ::core };

        let generics = &self.generics;
        let block = &self.block;
        let true_output = match &self.ret {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, ty) => quote! { #ty },
        };

        let mut args_tuple: Punctuated<_, Token![,]> = self
            .args
            .pairs()
            .map(|pair| pair.value().ty.clone())
            .collect();

        if args_tuple.is_empty() {
            args_tuple.push( parse_quote!{ () });
        } else if !args_tuple.trailing_punct() {
            args_tuple.push_punct(Token![,](Span::call_site()));
        }

        let inner_args: Punctuated<_, Token![,]> = self
            .args
            .pairs()
            .map(|pair| {
                let OverloadArg { name, colon: _, ty } = pair.value();
                quote!(#name: #ty)
            })
            .collect();

        let args_expansion: Punctuated<_, Token![,]> = self
            .args
            .pairs()
            .enumerate()
            .map(|(i, _)| {
                let i = Literal::usize_unsuffixed(i);
                quote!(args.#i)
            })
            .collect();

        let output;
        let inner_call;
        let inner_async;

        if let Asyncness::Async {
            async_token: _,
            not_sendness,
        } = self.asyncness
        {
            let sendness = if not_sendness.is_some() {
                quote!()
            } else {
                quote! { + #core_path::marker::Send }
            };

            inner_async = quote! { async };

            #[cfg(not(feature = "impl_futures"))]
            {
                #[cfg(feature = "std")]
                let box_path = quote! { ::std::boxed::Box };

                #[cfg(not(feature = "std"))]
                let box_path = quote! { ::alloc::boxed::Box };

                output = quote! {
                    #core_path::pin::Pin<#box_path<dyn #core_path::future::Future<Output = #true_output> #sendness>>
                };
                inner_call = quote! {
                    #box_path::pin(inner(#args_expansion))
                };
            }

            #[cfg(feature = "impl_futures")]
            {
                output = quote! {
                    impl #core_path::future::Future<Output = #true_output> #sendness
                };
                inner_call = quote! {
                    inner(#args_expansion)
                };
            }
        } else {
            inner_async = quote!();
            output = true_output.clone();
            inner_call = quote! {
                inner(#args_expansion)
            };
        }

        quote! {
            impl #generics #core_path::ops::FnOnce<(#args_tuple)> for #name {
                type Output = #output;

                extern "rust-call" fn call_once(self, args: (#args_tuple)) -> Self::Output {
                    #inner_async fn inner #generics (#inner_args) -> #true_output #block
                    #inner_call
                }
            }

            impl #generics #core_path::ops::FnMut<(#args_tuple)> for #name {
                extern "rust-call" fn call_mut(&mut self, args: (#args_tuple)) -> Self::Output {
                    #name.call_once(args)
                }
            }

            impl #generics #core_path::ops::Fn<(#args_tuple)> for #name {
                extern "rust-call" fn call(&self, args: (#args_tuple)) -> Self::Output {
                    #name.call_once(args)
                }
            }
        }
    }
}

#[allow(dead_code)]
struct OverloadedFn {
    visibility: Visibility,
    fn_token: Token![fn],
    name: Ident,
    brace: Brace,
    overloads: Punctuated<Overload, Token![;]>,
}

impl Parse for OverloadedFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility = input.parse()?;
        let fn_token = input.parse()?;
        let name = input.parse()?;

        let content;
        let brace = braced!(content in input);
        let overloads = Punctuated::parse_terminated(&content)?;

        Ok(Self {
            visibility,
            fn_token,
            name,
            brace,
            overloads,
        })
    }
}

#[proc_macro]
pub fn fn_overloads(input: TokenStream) -> TokenStream {
    let OverloadedFn {
        visibility,
        fn_token: _,
        name,
        brace: _,
        overloads,
    } = parse_macro_input!(input as OverloadedFn);

    let mut expanded = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        #visibility struct #name;
    };

    expanded.append_all(overloads.pairs().map(|pair| pair.value().to_tokens(&name)));

    expanded.into()
}