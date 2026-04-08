//! Proc macros for `telemetry-safe-tracing`.
//!
//! `safe_instrument` will live here so the public tracing crate can stay a
//! normal library and still expose helper types alongside the attribute macro.

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::spanned::Spanned;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Error, Expr, Ident, ItemFn, Result, Token, parenthesized, parse_macro_input,
};

#[proc_macro_attribute]
pub fn safe_instrument(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as InstrumentArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    match expand_safe_instrument(args, item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_safe_instrument(args: InstrumentArgs, item_fn: ItemFn) -> Result<proc_macro2::TokenStream> {
    let attr = args.expand()?;

    Ok(quote! {
        #[::telemetry_safe_tracing::tracing::instrument(#attr)]
        #item_fn
    })
}

struct InstrumentArgs {
    args: Punctuated<InstrumentArg, Token![,]>,
}

impl Parse for InstrumentArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

impl InstrumentArgs {
    fn expand(self) -> Result<proc_macro2::TokenStream> {
        // `instrument` defaults to recording every argument via `Debug`, which is
        // exactly the ambient escape hatch this macro exists to remove.
        let mut expanded = vec![quote! { skip_all }];
        for arg in self.args {
            if let Some(tokens) = arg.expand()? {
                expanded.push(tokens);
            }
        }

        Ok(quote! { #(#expanded),* })
    }
}

enum InstrumentArg {
    Flag(Ident),
    NameValue { name: Ident, value: Expr },
    List { name: Ident, tokens: proc_macro2::TokenStream },
}

impl Parse for InstrumentArg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;

        if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            let tokens = content.parse()?;
            return Ok(Self::List { name, tokens });
        }

        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value: Expr = input.parse()?;
            return Ok(Self::NameValue { name, value });
        }

        Ok(Self::Flag(name))
    }
}

impl InstrumentArg {
    fn expand(self) -> Result<Option<proc_macro2::TokenStream>> {
        match self {
            Self::Flag(name) => match name.to_string().as_str() {
                "skip_all" => Ok(None),
                "err" | "ret" => Err(Error::new(
                    name.span(),
                    "`err` and `ret` are intentionally unsupported in safe_instrument; record explicit safe fields instead",
                )),
                _ => Err(Error::new(
                    name.span(),
                    "unsupported safe_instrument flag; only `skip_all`, `skip(...)`, `name`, `level`, `target`, and `fields(...)` are currently supported",
                )),
            },
            Self::NameValue { name, value } => match name.to_string().as_str() {
                "name" | "level" | "target" => Ok(Some(quote! { #name = #value })),
                "err" | "ret" => Err(Error::new(
                    name.span(),
                    "`err` and `ret` are intentionally unsupported in safe_instrument; record explicit safe fields instead",
                )),
                _ => Err(Error::new(
                    name.span(),
                    "unsupported safe_instrument option; only `name`, `level`, `target`, `skip(...)`, `skip_all`, and `fields(...)` are currently supported",
                )),
            },
            Self::List { name, tokens } => match name.to_string().as_str() {
                // `safe_instrument` already forces `skip_all`, so forwarding a
                // partial skip list would only add confusing, redundant syntax.
                "skip" => Ok(None),
                "fields" => {
                    let fields = syn::parse2::<FieldArgs>(tokens)?;
                    let expanded = fields.expand()?;
                    Ok(Some(quote! { fields(#expanded) }))
                }
                _ => Err(Error::new(
                    name.span(),
                    "unsupported safe_instrument list; only `skip(...)` and `fields(...)` are currently supported",
                )),
            },
        }
    }
}

struct FieldArgs {
    fields: Punctuated<FieldArg, Token![,]>,
}

impl Parse for FieldArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            fields: Punctuated::parse_terminated(input)?,
        })
    }
}

impl FieldArgs {
    fn expand(self) -> Result<proc_macro2::TokenStream> {
        let mut expanded = Vec::with_capacity(self.fields.len());
        for field in self.fields {
            expanded.push(field.expand()?);
        }

        Ok(quote! { #(#expanded),* })
    }
}

struct FieldArg {
    name: proc_macro2::TokenStream,
    kind: FieldValueKind,
}

impl Parse for FieldArg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut name = proc_macro2::TokenStream::new();
        while !input.peek(Token![=]) {
            if input.is_empty() {
                return Err(input.error("field entries must use `name = %expr`"));
            }

            let tt: TokenTree = input.parse()?;
            name.extend(std::iter::once(tt));
        }

        let _: Token![=] = input.parse()?;

        let kind = if input.peek(Token![%]) {
            let _: Token![%] = input.parse()?;
            FieldValueKind::Display(input.parse()?)
        } else if input.peek(Token![?]) {
            let mark: Token![?] = input.parse()?;
            let _expr: Expr = input.parse()?;
            return Err(Error::new(
                mark.span,
                "`?expr` is intentionally unsupported in safe_instrument; use `%expr` with a ToTelemetry value instead",
            ));
        } else {
            let value: Expr = input.parse()?;
            return Err(Error::new(
                value.span(),
                "field entries must use `%expr`; implicit value formatting is intentionally unsupported",
            ));
        };

        Ok(Self { name, kind })
    }
}

impl FieldArg {
    fn expand(self) -> Result<proc_macro2::TokenStream> {
        if self.name.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "field name cannot be empty",
            ));
        }

        let name = self.name;
        match self.kind {
            FieldValueKind::Display(expr) => Ok(quote! {
                #name = %::telemetry_safe_tracing::telemetry(&(#expr))
            }),
        }
    }
}

enum FieldValueKind {
    Display(Expr),
}
