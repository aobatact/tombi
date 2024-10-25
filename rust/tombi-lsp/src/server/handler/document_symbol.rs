use crate::{server::backend::Backend, toml};
use ast::AstNode;
use text_position::TextPosition;
use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, MessageType, Position, Range,
    SymbolKind,
};

pub async fn handle_document_symbol(
    backend: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
    let source = toml::try_load(&params.text_document.uri)?;

    let p = parser::parse(&source);
    let symbol_infos = ast::Root::cast(p.into_syntax_node())
        .map(|root| root.into_symbols(&source))
        .unwrap_or_default();

    backend
        .client
        .log_message(MessageType::INFO, format!("Symbols: {symbol_infos:#?}"))
        .await;

    Ok(Some(DocumentSymbolResponse::Nested(symbol_infos)))
}

trait IntoSymbols {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol>;
}

trait IntoSymbolsWithChildren {
    fn into_symbols_with_childen(
        self,
        source: &str,
        children: Option<Vec<DocumentSymbol>>,
    ) -> Vec<DocumentSymbol>;
}

trait IntoKeysSymbols {
    fn into_keys_symbols(
        self,
        source: &str,
        kind: tower_lsp::lsp_types::SymbolKind,
        children: Option<Vec<DocumentSymbol>>,
    ) -> Vec<DocumentSymbol>;
}

impl IntoSymbols for ast::Root {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        self.items()
            .map(|item| item.into_symbols(source))
            .flatten()
            .collect()
    }
}

impl IntoSymbols for ast::RootItem {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        match self {
            Self::Table(table) => table.into_symbols(source),
            Self::ArrayOfTable(array_of_table) => array_of_table.into_symbols(source),
            Self::KeyValue(kv) => kv.into_symbols(source),
        }
    }
}

impl IntoSymbols for ast::Table {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        let childlens = self
            .key_values()
            .map(|kv| kv.into_symbols(source))
            .flatten()
            .collect::<Vec<_>>();

        let childrens = if childlens.is_empty() {
            None
        } else {
            Some(childlens)
        };

        self.header()
            .map(|header| header.into_keys_symbols(source, SymbolKind::OBJECT, childrens))
            .unwrap_or_default()
    }
}

impl IntoSymbols for ast::ArrayOfTable {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        let childlens = self
            .key_values()
            .map(|kv| kv.into_symbols(source))
            .flatten()
            .collect::<Vec<_>>();

        let childrens = if childlens.is_empty() {
            None
        } else {
            Some(childlens)
        };

        self.header()
            .map(|header| header.into_keys_symbols(source, SymbolKind::OBJECT, childrens))
            .unwrap_or_default()
    }
}

impl IntoSymbols for ast::KeyValue {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        if let Some(keys) = self.keys() {
            let children = self.value().map(|value| value.into_symbols(source));
            keys.into_keys_symbols(source, SymbolKind::VARIABLE, children)
        } else {
            vec![]
        }
    }
}

impl IntoKeysSymbols for ast::Keys {
    fn into_keys_symbols(
        self,
        source: &str,
        kind: tower_lsp::lsp_types::SymbolKind,
        children: Option<Vec<DocumentSymbol>>,
    ) -> Vec<DocumentSymbol> {
        self.keys()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .fold(children.unwrap_or_default(), |children, key| {
                let start_pos =
                    TextPosition::from_source(source, key.syntax().text_range().start());
                let end_pos = TextPosition::from_source(source, key.syntax().text_range().end());
                let range = Range {
                    start: Position {
                        line: start_pos.line(),
                        character: start_pos.column(),
                    },
                    end: Position {
                        line: end_pos.line(),
                        character: end_pos.column(),
                    },
                };

                let symbols = vec![
                    #[allow(deprecated)]
                    DocumentSymbol {
                        name: key.syntax().text().to_string(),
                        kind,
                        tags: None,
                        range,
                        selection_range: range,
                        children: Some(children.clone()),
                        deprecated: None,
                        detail: None,
                    },
                ];

                symbols
            })
    }
}

impl IntoSymbols for ast::Value {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        match self {
            Self::BasicString(_)
            | Self::LiteralString(_)
            | Self::MultiLineBasicString(_)
            | Self::MultiLineLiteralString(_)
            | Self::IntegerBin(_)
            | Self::IntegerOct(_)
            | Self::IntegerDec(_)
            | Self::IntegerHex(_)
            | Self::Float(_)
            | Self::Boolean(_)
            | Self::OffsetDateTime(_)
            | Self::LocalDateTime(_)
            | Self::LocalDate(_)
            | Self::LocalTime(_) => vec![],
            Self::Array(array) => array.into_symbols(source),
            Self::InlineTable(inline_table) => inline_table.into_symbols(source),
        }
    }
}

impl IntoSymbols for ast::Array {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        self.elements()
            .map(|element| element.into_symbols(source))
            .flatten()
            .collect()
    }
}

impl IntoSymbols for ast::InlineTable {
    fn into_symbols(self, source: &str) -> Vec<DocumentSymbol> {
        self.elements()
            .map(|element| element.into_symbols(source))
            .flatten()
            .collect()
    }
}

// not document this function
#[allow(dead_code)]
fn debug_document_symbol() -> Vec<DocumentSymbol> {
    vec![
        #[allow(deprecated)]
        DocumentSymbol {
            name: "debug".to_string(),
            kind: tower_lsp::lsp_types::SymbolKind::VARIABLE,
            tags: None,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            selection_range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            children: None,
            deprecated: None,
            detail: None,
        },
    ]
}
