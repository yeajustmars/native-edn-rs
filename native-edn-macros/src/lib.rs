use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitFloat, LitInt, LitStr, Token, parse_macro_input, token};

// 1. Compile-time Representation
enum EdnAst {
    Integer(i64),
    Float(f64),
    String(String),
    Keyword(String),
    Vector(Vec<EdnAst>),
    Map(Vec<(EdnAst, EdnAst)>),
    Set(Vec<EdnAst>),
    Uuid(String),
    Tagged(String, Box<EdnAst>),
}

// 2. Parsing the EDN Syntax
impl Parse for EdnAst {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Skip optional commas (Clojure treats them as whitespace)
        while input.peek(Token![,]) {
            let _ = input.parse::<Token![,]>()?;
        }

        // Keywords (:x, :user/id)
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let kw = parse_clojure_ident(input)?;
            return Ok(EdnAst::Keyword(kw));
        }

        // Tagged Literals and Sets (#uuid, #my-tag, #{1 2})
        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;

            // Set: #{...}
            if input.peek(token::Brace) {
                let content;
                syn::braced!(content in input);
                let mut items = Vec::new();
                while !content.is_empty() {
                    items.push(content.parse()?);
                }
                return Ok(EdnAst::Set(items));
            }

            // Tagged literal: #uuid "..." or #my/tag [...]
            let tag = parse_clojure_ident(input)?;
            let next_val: EdnAst = input.parse()?;

            if tag == "uuid"
                && let EdnAst::String(s) = next_val
            {
                return Ok(EdnAst::Uuid(s));
            }
            return Ok(EdnAst::Tagged(tag, Box::new(next_val)));
        }

        // Vectors: [...]
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(content.parse()?);
            }
            return Ok(EdnAst::Vector(items));
        }

        // Maps: {...}
        if input.peek(token::Brace) {
            let content;
            syn::braced!(content in input);
            let mut pairs = Vec::new();
            while !content.is_empty() {
                let k: EdnAst = content.parse()?;
                let v: EdnAst = content.parse()?;
                pairs.push((k, v));
            }
            return Ok(EdnAst::Map(pairs));
        }

        // Strings
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            return Ok(EdnAst::String(lit.value()));
        }

        // Floats and Ints
        if input.peek(LitFloat) {
            let lit: LitFloat = input.parse()?;
            return Ok(EdnAst::Float(lit.base10_parse()?));
        }
        if input.peek(LitInt) {
            let lit: LitInt = input.parse()?;
            return Ok(EdnAst::Integer(lit.base10_parse()?));
        }

        Err(input.error("Unsupported or invalid EDN syntax"))
    }
}

// Helper to eagerly consume '-', '/', and idents for Clojure names
fn parse_clojure_ident(input: ParseStream) -> syn::Result<String> {
    let mut name = String::new();
    while !input.is_empty() {
        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            name.push_str(&ident.to_string());
        } else if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            name.push('-');
        } else if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            name.push('/');
        } else {
            break; // Not a valid identifier character
        }
    }
    Ok(name)
}

// 3. Code Generation (Converting Compile-time AST to Run-time tokens)
impl ToTokens for EdnAst {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ts = match self {
            EdnAst::Integer(i) => quote! { native_edn::Edn::Integer(#i) },
            EdnAst::Float(f) => quote! { native_edn::Edn::Float(native_edn::EdnFloat(#f)) },
            EdnAst::String(s) => quote! { native_edn::Edn::String(#s.to_string()) },
            EdnAst::Keyword(k) => quote! { native_edn::Edn::Keyword(#k.to_string()) },
            EdnAst::Uuid(u) => quote! {
                native_edn::Edn::Uuid(uuid::Uuid::parse_str(#u).expect("Invalid UUID syntax in edn!"))
            },
            EdnAst::Vector(vec) => quote! {
                native_edn::Edn::Vector(vec![ #(#vec),* ])
            },
            EdnAst::Set(set) => quote! {
                native_edn::Edn::Set(std::collections::BTreeSet::from([ #(#set),* ]))
            },
            EdnAst::Map(pairs) => {
                let keys = pairs.iter().map(|(k, _)| k);
                let vals = pairs.iter().map(|(_, v)| v);
                quote! {
                    native_edn::Edn::Map(std::collections::BTreeMap::from([
                        #( (#keys, #vals) ),*
                    ]))
                }
            }
            EdnAst::Tagged(tag, val) => quote! {
                native_edn::Edn::Tagged(#tag.to_string(), Box::new(#val))
            },
        };
        tokens.extend(ts);
    }
}

#[proc_macro]
pub fn edn(input: TokenStream) -> TokenStream {
    // 1. Parse the Rust TokenStream into our EDN AST
    let parsed_edn = parse_macro_input!(input as EdnAst);

    // 2. Generate the Rust code that will run at runtime
    let expanded = quote! { #parsed_edn };

    TokenStream::from(expanded)
}
