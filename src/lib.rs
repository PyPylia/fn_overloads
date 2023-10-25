use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TokenStream2, Span};
use quote::{quote, TokenStreamExt};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Block, Generics, Ident, Result, ReturnType, Token, Type, Visibility,
};

#[allow(dead_code)]
//#[derive(Debug)]
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
    asyncness: Option<Token![async]>,
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

        if !args_tuple.trailing_punct() {
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

        if self.asyncness.is_some() {
            #[cfg(not(feature = "impl_futures"))]
            {
                #[cfg(feature = "std")]
                let box_path = quote! { ::std::boxed::Box };
        
                #[cfg(not(feature = "std"))]
                let box_path = quote! { ::alloc::boxed::Box };

                output = quote! {
                    ::core::pin::Pin<#box_path<dyn ::core::future::Future<Output = #true_output>>>
                };
                inner_call = quote! {
                    #box_path::pin(async move { inner(#args_expansion) })
                };
            }

            #[cfg(feature = "impl_futures")]
            {
                output = quote! {
                    impl ::core::future::Future<Output = #true_output>
                };
                inner_call = quote! {
                    async move { inner(#args_expansion) }
                };
            }
        } else {
            output = true_output.clone();
            inner_call = quote! {
                inner(#args_expansion)
            };
        }

        quote! {
            impl #generics ::core::ops::FnOnce<(#args_tuple)> for #name {
                type Output = #output;

                extern "rust-call" fn call_once(self, args: (#args_tuple)) -> Self::Output {
                    fn inner #generics (#inner_args) -> #true_output #block
                    #inner_call
                }
            }

            impl #generics ::core::ops::FnMut<(#args_tuple)> for #name {
                extern "rust-call" fn call_mut(&mut self, args: (#args_tuple)) -> Self::Output {
                    #name.call_once(args)
                }
            }

            impl #generics ::core::ops::Fn<(#args_tuple)> for #name {
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
