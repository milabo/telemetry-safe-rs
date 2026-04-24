use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Fields, LitStr, Result, Token,
};

#[proc_macro_derive(ToTelemetry, attributes(telemetry))]
pub fn derive_to_telemetry(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => expand_struct(ident, data)?,
        Data::Enum(data) => expand_enum(data)?,
        Data::Union(data) => {
            return Err(Error::new(
                data.union_token.span(),
                "ToTelemetry cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics ::telemetry_safe::ToTelemetry for #ident #ty_generics #where_clause {
            fn fmt_telemetry(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> ::std::fmt::Result {
                #body
            }
        }
    })
}

fn expand_struct(ident: &syn::Ident, data: &DataStruct) -> Result<proc_macro2::TokenStream> {
    match &data.fields {
        Fields::Named(fields) => {
            let mut field_exprs = Vec::new();
            for field in &fields.named {
                let attr = parse_field_attr(&field.attrs).transpose()?;
                if matches!(attr, Some(FieldAttr::Skip)) {
                    continue;
                }

                let name = field.ident.as_ref().expect("named field");
                let key = LitStr::new(&name.to_string(), name.span());
                let value = field_expr(field, quote! { self.#name }, attr)?;
                field_exprs.push(quote! {
                    ds.field(#key, &#value);
                });
            }

            Ok(quote! {
                let mut ds = f.debug_struct(stringify!(#ident));
                #(#field_exprs)*
                ds.finish()
            })
        }
        Fields::Unnamed(fields) => {
            let mut field_exprs = Vec::new();
            for (index, field) in fields.unnamed.iter().enumerate() {
                let attr = parse_field_attr(&field.attrs).transpose()?;
                if matches!(attr, Some(FieldAttr::Skip)) {
                    continue;
                }

                let accessor = syn::Index::from(index);
                let value = field_expr(field, quote! { self.#accessor }, attr)?;
                field_exprs.push(quote! {
                    ds.field(&#value);
                });
            }

            Ok(quote! {
                let mut ds = f.debug_tuple(stringify!(#ident));
                #(#field_exprs)*
                ds.finish()
            })
        }
        Fields::Unit => Ok(quote! {
            f.write_str(stringify!(#ident))
        }),
    }
}

fn expand_enum(data: &DataEnum) -> Result<proc_macro2::TokenStream> {
    let arms = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Named(fields) => {
                    let mut bindings = Vec::new();
                    let mut formatter = Vec::new();
                    for field in &fields.named {
                        let attr = parse_field_attr(&field.attrs).transpose()?;
                        let name = field.ident.as_ref().expect("named field");

                        if matches!(attr, Some(FieldAttr::Skip)) {
                            bindings.push(quote! { #name: _ });
                            continue;
                        }

                        if !field_attr_requires_binding(attr.as_ref()) {
                            // Fields that never read the matched value must bind `_`,
                            // otherwise enum patterns leak `unused variable` warnings
                            // into downstream crates despite being intentionally ignored.
                            bindings.push(quote! { #name: _ });
                        } else {
                            bindings.push(quote! { #name });
                        }

                        let key = LitStr::new(&name.to_string(), name.span());
                        let value = field_expr(field, quote! { #name }, attr)?;
                        formatter.push(quote! {
                            ds.field(#key, &#value);
                        });
                    }

                    Ok(quote! {
                        Self::#ident { #(#bindings),* } => {
                            let mut ds = f.debug_struct(stringify!(#ident));
                            #(#formatter)*
                            ds.finish()
                        }
                    })
                }
                Fields::Unnamed(fields) => {
                    let mut bindings = Vec::new();
                    let mut formatter = Vec::new();
                    for (index, field) in fields.unnamed.iter().enumerate() {
                        let attr = parse_field_attr(&field.attrs).transpose()?;
                        let binding = syn::Ident::new(&format!("field_{index}"), ident.span());

                        if matches!(attr, Some(FieldAttr::Skip)) {
                            bindings.push(quote! { _ });
                            continue;
                        }

                        if !field_attr_requires_binding(attr.as_ref()) {
                            bindings.push(quote! { _ });
                        } else {
                            bindings.push(quote! { #binding });
                        }

                        let value = field_expr(field, quote! { #binding }, attr)?;
                        formatter.push(quote! {
                            ds.field(&#value);
                        });
                    }

                    Ok(quote! {
                        Self::#ident(#(#bindings),*) => {
                            let mut ds = f.debug_tuple(stringify!(#ident));
                            #(#formatter)*
                            ds.finish()
                        }
                    })
                }
                Fields::Unit => Ok(quote! {
                    Self::#ident => f.write_str(stringify!(#ident))
                }),
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        match self {
            #(#arms),*
        }
    })
}

fn field_expr(
    field: &syn::Field,
    accessor: proc_macro2::TokenStream,
    attr: Option<FieldAttr>,
) -> Result<proc_macro2::TokenStream> {
    match attr {
        Some(FieldAttr::Display(format)) => {
            // Keep the escape hatch intentionally narrow: a fixed replacement
            // string or a single `{}` placeholder. Allowing the full format
            // language would hide too much policy behind proc-macro magic.
            match format {
                DisplayFormat::Literal(literal) => Ok(quote! {
                    ::std::format_args!("{}", #literal)
                }),
                DisplayFormat::Interpolated(format) => Ok(quote! {
                    ::std::format_args!(#format, #accessor)
                }),
            }
        }
        Some(FieldAttr::Skip) | None => {
            let ty = &field.ty;
            Ok(quote! {{
                let value: &#ty = &#accessor;
                ::telemetry_safe::telemetry_debug(value)
            }})
        }
    }
}

enum FieldAttr {
    Display(DisplayFormat),
    Skip,
}

enum DisplayFormat {
    Literal(LitStr),
    Interpolated(LitStr),
}

fn field_attr_requires_binding(attr: Option<&FieldAttr>) -> bool {
    !matches!(
        attr,
        Some(FieldAttr::Skip | FieldAttr::Display(DisplayFormat::Literal(_)))
    )
}

fn parse_field_attr(attrs: &[Attribute]) -> Option<Result<FieldAttr>> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("telemetry"))
        .map(parse_single_field_attr)
}

fn parse_single_field_attr(attr: &Attribute) -> Result<FieldAttr> {
    attr.parse_args_with(|input: syn::parse::ParseStream<'_>| {
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "skip" {
                if !input.is_empty() {
                    return Err(input.error("unexpected tokens after skip"));
                }
                return Ok(FieldAttr::Skip);
            }

            return Err(Error::new(ident.span(), "unsupported telemetry attribute"));
        }

        let format: Expr = input.parse()?;
        if !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            if !input.is_empty() {
                return Err(input.error("expected a single format string or `skip`"));
            }
        }

        match format {
            Expr::Lit(expr_lit) => match expr_lit.lit {
                syn::Lit::Str(lit) => Ok(FieldAttr::Display(parse_display_format(lit)?)),
                other => Err(Error::new(other.span(), "expected string literal")),
            },
            other => Err(Error::new(other.span(), "expected string literal")),
        }
    })
}

fn parse_display_format(format: LitStr) -> Result<DisplayFormat> {
    let value = format.value();
    let placeholder_count = value.matches("{}").count();

    // We intentionally reject the broader formatting mini-language so users
    // can tell at a glance whether a field is redacted entirely or routes
    // through exactly one curated Display representation.
    if value.replace("{}", "").contains(['{', '}']) {
        return Err(Error::new(
            format.span(),
            "only a fixed string or a single `{}` placeholder is supported",
        ));
    }

    match placeholder_count {
        0 => Ok(DisplayFormat::Literal(format)),
        1 => Ok(DisplayFormat::Interpolated(format)),
        _ => Err(Error::new(
            format.span(),
            "only a fixed string or a single `{}` placeholder is supported",
        )),
    }
}
