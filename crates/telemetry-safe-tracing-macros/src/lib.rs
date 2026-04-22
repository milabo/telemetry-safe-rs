//! Proc macros for `telemetry-safe-tracing`.
//!
//! `safe_instrument` will live here so the public tracing crate can stay a
//! normal library and still expose helper types alongside the attribute macro.

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Error, Expr, Ident, ItemFn, Result, ReturnType, Token, parenthesized, parse_macro_input,
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

fn expand_safe_instrument(
    args: InstrumentArgs,
    item_fn: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    let config = args.expand()?;
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item_fn;

    if config.record_err && sig.output == ReturnType::Default {
        return Err(Error::new(
            sig.ident.span(),
            "`err` requires a `Result`-returning function",
        ));
    }

    let record_ret_enabled = config.record_ret;
    let record_err_enabled = config.record_err;
    let instrument_attr = config.instrument_attr();
    let record_ret = if record_ret_enabled {
        Some(quote! {
            ::telemetry_safe_tracing::__private::record_ret(
                &__telemetry_safe_span,
                &__telemetry_safe_result,
            );
        })
    } else {
        None
    };
    let record_err = if record_err_enabled {
        Some(quote! {
            ::telemetry_safe_tracing::__private::record_err(
                &__telemetry_safe_span,
                &__telemetry_safe_result,
            );
        })
    } else {
        None
    };

    let body = if sig.asyncness.is_some() {
        quote! {
            let __telemetry_safe_span = ::telemetry_safe_tracing::tracing::Span::current();
            let __telemetry_safe_result = (async move #block).await;
            #record_ret
            #record_err
            __telemetry_safe_result
        }
    } else {
        quote! {
            let __telemetry_safe_span = ::telemetry_safe_tracing::tracing::Span::current();
            let __telemetry_safe_result = (|| #block)();
            #record_ret
            #record_err
            __telemetry_safe_result
        }
    };

    Ok(quote! {
        #(#attrs)*
        #[::telemetry_safe_tracing::tracing::instrument(#instrument_attr)]
        #vis #sig {
            #body
        }
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
    fn expand(self) -> Result<InstrumentConfig> {
        // `instrument` defaults to recording every argument via `Debug`, which is
        // exactly the ambient escape hatch this macro exists to remove.
        let mut config = InstrumentConfig {
            attr_args: vec![quote! { skip_all }],
            field_args: Vec::new(),
            record_ret: false,
            record_err: false,
        };
        for arg in self.args {
            arg.apply(&mut config)?;
        }

        Ok(config)
    }
}

struct InstrumentConfig {
    attr_args: Vec<proc_macro2::TokenStream>,
    field_args: Vec<proc_macro2::TokenStream>,
    record_ret: bool,
    record_err: bool,
}

impl InstrumentConfig {
    fn instrument_attr(mut self) -> proc_macro2::TokenStream {
        if self.record_ret {
            self.field_args
                .push(quote! { ret = ::telemetry_safe_tracing::tracing::field::Empty });
        }
        if self.record_err {
            self.field_args
                .push(quote! { err = ::telemetry_safe_tracing::tracing::field::Empty });
        }
        if !self.field_args.is_empty() {
            let field_args = self.field_args;
            self.attr_args.push(quote! { fields(#(#field_args),*) });
        }

        let attr_args = self.attr_args;
        quote! { #(#attr_args),* }
    }
}

enum InstrumentArg {
    Flag(Ident),
    NameValue {
        name: Ident,
        value: Expr,
    },
    List {
        name: Ident,
        tokens: proc_macro2::TokenStream,
    },
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
    fn apply(self, config: &mut InstrumentConfig) -> Result<()> {
        match self {
            Self::Flag(name) => match name.to_string().as_str() {
                "skip_all" => Ok(()),
                "ret" => {
                    config.record_ret = true;
                    Ok(())
                }
                "err" => {
                    config.record_err = true;
                    Ok(())
                }
                _ => Err(Error::new(
                    name.span(),
                    "unsupported safe_instrument flag; only `skip_all`, `skip(...)`, `name`, `level`, `target`, `ret`, `err`, and `fields(...)` are currently supported",
                )),
            },
            Self::NameValue { name, value } => match name.to_string().as_str() {
                "name" | "level" | "target" => {
                    config.attr_args.push(quote! { #name = #value });
                    Ok(())
                }
                _ => Err(Error::new(
                    name.span(),
                    "unsupported safe_instrument option; only `name`, `level`, `target`, `skip(...)`, `skip_all`, `ret`, `err`, and `fields(...)` are currently supported",
                )),
            },
            Self::List { name, tokens } => match name.to_string().as_str() {
                // `safe_instrument` already forces `skip_all`, so forwarding a
                // partial skip list would only add confusing, redundant syntax.
                "skip" => Ok(()),
                "fields" => {
                    let fields = syn::parse2::<FieldArgs>(tokens)?;
                    config.field_args.extend(fields.expand()?);
                    Ok(())
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
    fn expand(self) -> Result<Vec<proc_macro2::TokenStream>> {
        let mut expanded = Vec::with_capacity(self.fields.len());
        for field in self.fields {
            expanded.push(field.expand()?);
        }

        Ok(expanded)
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
