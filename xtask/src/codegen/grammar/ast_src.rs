use quote::{format_ident, quote};

#[derive(Default, Debug)]
pub(crate) struct AstSrc {
    pub(crate) tokens: Vec<String>,
    pub(crate) nodes: Vec<AstNodeSrc>,
    pub(crate) enums: Vec<AstEnumSrc>,
}

#[derive(Debug)]
pub(crate) struct AstNodeSrc {
    #[allow(dead_code)]
    pub(crate) doc: Vec<String>,
    pub(crate) name: String,
    pub(crate) traits: Vec<String>,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct AstEnumSrc {
    #[allow(dead_code)]
    pub(crate) doc: Vec<String>,
    pub(crate) name: String,
    pub(crate) traits: Vec<String>,
    pub(crate) variants: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Field {
    Token(String),
    Node {
        name: String,
        ty: String,
        cardinality: Cardinality,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Cardinality {
    Optional,
    Many,
}

impl Field {
    pub fn is_many(&self) -> bool {
        matches!(
            self,
            Field::Node {
                cardinality: Cardinality::Many,
                ..
            }
        )
    }
    pub fn token_kind(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            Field::Token(token) => match token.as_str() {
                "[[" | "]]" => Some(quote! { T![#token]}),
                token if token.chars().all(|s| matches!(s, 'a'..='z' | '_')) => {
                    let token: proc_macro2::TokenStream = token.to_uppercase().parse().unwrap();
                    Some(quote! {#token})
                }
                _ => {
                    let token: proc_macro2::TokenStream = token.parse().unwrap();
                    Some(quote! { T![#token] })
                }
            },
            _ => None,
        }
    }

    pub fn method_name(&self) -> String {
        match self {
            Field::Token(name) => match name.as_str() {
                "'{'" => "brace_start",
                "'}'" => "brace_end",
                "'['" => "bracket_start",
                "']'" => "bracket_end",
                "[[" => "double_bracket_start",
                "]]" => "double_bracket_end",
                "=" => "eq",
                "." => "dot",
                "," => "comma",
                _ => "token",
            },
            Field::Node { name, .. } => match name.as_str() {
                "root_items" => "items",
                _ => name,
            },
        }
        .to_owned()
    }

    pub fn ty(&self) -> proc_macro2::Ident {
        match self {
            Field::Token(_) => format_ident!("SyntaxToken"),
            Field::Node { ty, .. } => format_ident!("{}", ty),
        }
    }
}
