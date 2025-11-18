use proc_macro::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::*;
use syn::spanned::Spanned;
use syn::*;

mod kw {
    syn::custom_keyword!(level);
    syn::custom_keyword!(fields);
    syn::custom_keyword!(err);
    syn::custom_keyword!(ret);
}

#[derive(Default)]
struct TracedArgs {
    level: Option<Level>,
    fields: Option<Fields>,
    emit_error: Option<FormatMode>,
    emit_return: Option<FormatMode>,
}

impl Parse for TracedArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::level) {
                args.level = Some(input.parse()?);
            } else if lookahead.peek(kw::fields) {
                args.fields = Some(input.parse()?);
            } else if lookahead.peek(kw::err) {
                let _ = input.parse::<kw::err>()?;
                args.emit_error = Some(input.parse()?);
            } else if lookahead.peek(kw::ret) {
                let _ = input.parse::<kw::ret>()?;
                args.emit_return = Some(input.parse()?);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(args)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ToTokens for Level {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt = match self {
            Level::Trace => quote! { Level::Trace },
            Level::Debug => quote! { Level::Debug },
            Level::Info => quote! { Level::Info },
            Level::Warn => quote! { Level::Warn },
            Level::Error => quote! { Level::Error },
        };

        tokens.extend(tt);
    }
}

impl Parse for Level {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<kw::level>()?;
        input.parse::<Token![=]>()?;

        let lvl = input.parse::<Ident>()?;

        match lvl.to_string().to_lowercase().as_str() {
            "trace" => Ok(Level::Trace),
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warn" => Ok(Level::Warn),
            "error" => Ok(Level::Error),
            unk => Err(Error::new(lvl.span(), format!("found unrecognized level, {unk}"))),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Fields(pub(crate) Punctuated<Field, Token![,]>);

impl Parse for Fields {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::fields>();

        let content;
        let _ = syn::parenthesized!(content in input);

        let fields = content.parse_terminated(Field::parse, Token![,])?;

        Ok(Self(fields))
    }
}

#[derive(Clone)]
pub(crate) struct Field {
    pub(crate) name: Punctuated<Ident, Token![.]>,
    pub(crate) value: Option<Expr>,
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = Punctuated::parse_separated_nonempty_with(input, Ident::parse_any)?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { name, value })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatMode {
    #[default]
    Debug,
    Display,
}

impl Parse for FormatMode {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if !input.peek(syn::token::Paren) {
            return Ok(Self::default());
        }

        let content;
        let _ = syn::parenthesized!(content in input);

        let mode = if let Some(ident) = content.parse::<Option<Ident>>()? {
            match ident.to_string().as_str() {
                "Debug" => FormatMode::Debug,
                "Display" => FormatMode::Display,
                _ => return Err(syn::Error::new(ident.span(), "expected either `Debug` or `Display`")),
            }
        } else {
            return Err(syn::Error::new(content.span(), "expected either `Debug` or `Display`"));
        };

        Ok(mode)
    }
}

pub(crate) fn traced(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as TracedArgs);

    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { attrs, vis, sig, .. } = &input;

    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        ident,
        inputs,
        output,
        fn_token,
        generics:
            syn::Generics {
                params: gen_params,
                where_clause,
                lt_token,
                gt_token,
            },
        ..
    } = sig;

    let fn_signature = quote_spanned! { sig.span() =>
        #(#attrs)*
        #[allow(unused_must_use, reason = "auto-generated")]
        #vis #constness #asyncness #unsafety #abi #fn_token #ident
        #lt_token #gen_params #gt_token (#inputs) #output #where_clause
    };

    // Install a fake return statement as the first thing in the function
    // body, so that we eagerly infer that the return type is what we
    // declared in the async fn signature.
    //
    // The `#[allow(..)]` is given because the return statement is
    // unreachable, but does affect inference, so it needs to be written
    // exactly that way for it to do its magic.
    let output_ty = match output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => ty.into_token_stream(),
    };

    let fake_return_edge = quote! {
        #[allow(
            unknown_lints,
            unreachable_code,
            clippy::diverging_sub_expression,
            clippy::empty_loop,
            clippy::let_unit_value,
            clippy::let_with_type_underscore,
            clippy::needless_return,
            clippy::unreachable
        )]
        if false {
            let __query_attr_fake_return: #output_ty = loop {};
            return __query_attr_fake_return;
        }
    };

    let block = build_block(&args, &input);

    quote_spanned! { sig.span() =>
        #fn_signature {
            #fake_return_edge
            { #block }
        }
    }
    .into()
}

fn build_block(args: &TracedArgs, input: &ItemFn) -> proc_macro2::TokenStream {
    let ItemFn { block, sig, .. } = &input;
    let Signature { ident, .. } = sig;

    let level = if let Some(level) = &args.level {
        quote_spanned! { level.span() => ::libftrace::#level }
    } else {
        quote_spanned! { input.span() => ::libftrace::Level::Info }
    };

    let fields = if let Some(fields) = &args.fields {
        let mut tt = quote! {};

        for field in &fields.0 {
            let key = &field.name;
            let value = if let Some(value) = &field.value {
                quote! { #value }
            } else {
                quote! { #key }
            };

            tt.extend(quote! {
                .with_field(stringify!(#key), format!("{:?}", #value))
            });
        }

        tt
    } else {
        quote! {}
    };

    let target = quote! { concat!(module_path!(), "::", stringify!(#ident)) };

    let enter_span_guard = quote! {
        let __guard = libftrace::with_subscriber(|s| {
            s.enter_span(
                libftrace::SpanMetadata::new(#target, #level)
                    #fields
            )
        })
    };

    let err_event = match args.emit_error {
        Some(FormatMode::Display) => quote! {
            ::libftrace::error!(#target, error = format!("{e}"))
        },
        None | Some(FormatMode::Debug) => quote! {
            ::libftrace::error!(#target, error = format!("{e:?}"))
        },
    };

    let ret_event = match args.emit_return {
        Some(FormatMode::Display) => quote! {
            ::libftrace::event!(level: #level, #target, ret = format!("{x}"))
        },
        None | Some(FormatMode::Debug) => quote! {
            ::libftrace::event!(level: #level, #target, ret = format!("{x:?}"))
        },
    };

    let block_result_emit = match (args.emit_error, args.emit_return) {
        (Some(_), Some(_)) => quote! {
            #[allow(clippy::redundant_closure_call)]
            match (move || #block)() {
                #[allow(clippy::unit_arg)]
                Ok(x) => {
                    #ret_event;
                    Ok(x)
                },
                Err(e) => {
                    #err_event;
                    Err(e)
                }
            }
        },
        (Some(_), None) => quote! {
            #[allow(clippy::redundant_closure_call)]
            match (move || #block)() {
                #[allow(clippy::unit_arg)]
                Ok(x) => Ok(x),
                Err(e) => {
                    #err_event;
                    Err(e)
                }
            }
        },
        (None, Some(_)) => quote! {
            #[allow(clippy::redundant_closure_call)]
            let x = (move || #block)();
            #ret_event;

            x
        },
        (None, None) => quote! {
            #block
        },
    };

    quote! {
        #enter_span_guard;

        #block_result_emit
    }
}
