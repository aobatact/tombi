pub mod algo;
mod generated;
pub(crate) mod support;

use config::TomlVersion;
pub use generated::*;
use itertools::Itertools;
use std::{fmt::Debug, marker::PhantomData};
use syntax::{SyntaxKind::*, T};

pub trait AstNode
where
    Self: Debug,
{
    fn leading_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::leading_comments(self.syntax().children_with_tokens())
    }

    fn tailing_comment(&self) -> Option<crate::Comment> {
        self.syntax().last_token().and_then(crate::Comment::cast)
    }

    fn can_cast(kind: syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: syntax::SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &syntax::SyntaxNode;
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: syntax::SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &syntax::SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: syntax::SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &syntax::SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

#[derive(Debug, Clone)]
pub enum TableOrArrayOfTable {
    Table(Table),
    ArrayOfTable(ArrayOfTable),
}

impl TableOrArrayOfTable {
    pub fn header(&self) -> Option<Keys> {
        match self {
            TableOrArrayOfTable::Table(table) => table.header(),
            TableOrArrayOfTable::ArrayOfTable(array_of_table) => array_of_table.header(),
        }
    }

    pub fn range(&self) -> text::Range {
        match self {
            TableOrArrayOfTable::Table(table) => table.range(),
            TableOrArrayOfTable::ArrayOfTable(array_of_table) => array_of_table.range(),
        }
    }
}

impl AstNode for TableOrArrayOfTable {
    #[inline]
    fn can_cast(kind: syntax::SyntaxKind) -> bool {
        Table::can_cast(kind) || ArrayOfTable::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: syntax::SyntaxNode) -> Option<Self> {
        if Table::can_cast(syntax.kind()) {
            Table::cast(syntax).map(TableOrArrayOfTable::Table)
        } else if ArrayOfTable::can_cast(syntax.kind()) {
            ArrayOfTable::cast(syntax).map(TableOrArrayOfTable::ArrayOfTable)
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &syntax::SyntaxNode {
        match self {
            TableOrArrayOfTable::Table(table) => table.syntax(),
            TableOrArrayOfTable::ArrayOfTable(array_of_table) => array_of_table.syntax(),
        }
    }
}

impl Root {
    pub fn begin_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::begin_dangling_comments(self.syntax().children_with_tokens())
    }

    pub fn end_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::end_dangling_comments(self.syntax().children_with_tokens())
    }

    pub fn dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::dangling_comments(self.syntax().children_with_tokens())
    }
}

impl Table {
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::leading_comments(self.syntax().children_with_tokens())
    }

    pub fn header_tailing_comment(&self) -> Option<crate::Comment> {
        support::tailing_comment(self.syntax().children_with_tokens(), T!(']'))
    }

    /// Returns an iterator over the subtables of this table.
    ///
    /// ```toml
    /// [foo]  # <- This is a self table
    /// [foo.bar]  # <- This is a subtable
    /// key = "value"
    ///
    /// [[foo.bar.baz]]  # <- This is also a subtable
    /// key = true
    /// ```
    pub fn subtables<'a>(&'a self) -> impl Iterator<Item = TableOrArrayOfTable> + 'a {
        support::next_siblings_nodes(self).take_while(|t: &TableOrArrayOfTable| {
            t.header()
                .unwrap()
                .to_string()
                .starts_with(&self.header().unwrap().to_string())
        })
    }

    pub fn parent_tables<'a>(&'a self) -> impl Iterator<Item = TableOrArrayOfTable> + 'a {
        support::prev_siblings_nodes(self).take_while(|t: &TableOrArrayOfTable| {
            self.header()
                .unwrap()
                .to_string()
                .starts_with(&t.header().unwrap().to_string())
        })
    }
}

impl ArrayOfTable {
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::leading_comments(self.syntax().children_with_tokens())
    }

    pub fn header_tailing_comment(&self) -> Option<crate::Comment> {
        support::tailing_comment(self.syntax().children_with_tokens(), T!("]]"))
    }
}

impl Array {
    #[inline]
    pub fn inner_begin_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('['))
                .skip(1),
        )
    }

    #[inline]
    pub fn inner_end_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::end_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .take_while(|node| node.kind() != T!(']')),
        )
    }

    #[inline]
    pub fn dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::dangling_comments(self.syntax().children_with_tokens())
    }

    #[inline]
    pub fn values_with_comma(&self) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        self.values()
            .zip_longest(support::children::<crate::Comma>(self.syntax()))
            .map(|value_with_comma| match value_with_comma {
                itertools::EitherOrBoth::Both(value, comma) => (value, Some(comma)),
                itertools::EitherOrBoth::Left(value) => (value, None),
                itertools::EitherOrBoth::Right(_) => unreachable!(),
            })
    }

    pub fn should_be_multiline(&self, toml_version: TomlVersion) -> bool {
        self.has_tailing_comma_after_last_value()
            || self.has_multiline_values(toml_version)
            || self.has_inner_comments()
    }

    pub fn has_tailing_comma_after_last_value(&self) -> bool {
        self.syntax()
            .children_with_tokens()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip_while(|item| item.kind() != T!(']'))
            .skip(1)
            .find(|item| !matches!(item.kind(), WHITESPACE | COMMENT | LINE_BREAK))
            .map_or(false, |it| it.kind() == T!(,))
    }

    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        self.values().any(|value| match value {
            crate::Value::Array(array) => array.should_be_multiline(toml_version),
            crate::Value::InlineTable(inline_table) => {
                inline_table.should_be_multiline(toml_version)
            }
            crate::Value::MultiLineBasicString(string) => {
                string.token().unwrap().text().contains('\n')
            }
            crate::Value::MultiLineLiteralString(string) => {
                string.token().unwrap().text().contains('\n')
            }
            _ => false,
        })
    }

    pub fn has_inner_comments(&self) -> bool {
        support::has_inner_comments(self.syntax().children_with_tokens(), T!('['), T!(']'))
    }
}

impl InlineTable {
    pub fn inner_begin_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| node.kind() != T!('{'))
                .skip(1),
        )
    }

    pub fn inner_end_dangling_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::end_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .take_while(|node| node.kind() != T!('}')),
        )
    }

    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        self.key_values()
            .zip_longest(support::children::<crate::Comma>(self.syntax()))
            .map(|value_with_comma| match value_with_comma {
                itertools::EitherOrBoth::Both(value, comma) => (value, Some(comma)),
                itertools::EitherOrBoth::Left(value) => (value, None),
                itertools::EitherOrBoth::Right(_) => unreachable!(),
            })
    }

    pub fn should_be_multiline(&self, toml_version: TomlVersion) -> bool {
        match toml_version {
            TomlVersion::V1_0_0 => false,
            TomlVersion::V1_1_0_Preview => {
                self.has_tailing_comma_after_last_value()
                    || self.has_multiline_values(toml_version)
                    || self.has_inner_comments()
            }
        }
    }

    pub fn has_tailing_comma_after_last_value(&self) -> bool {
        self.syntax()
            .children_with_tokens()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip_while(|item| item.kind() != T!('}'))
            .skip(1)
            .find(|item| !matches!(item.kind(), WHITESPACE | COMMENT | LINE_BREAK))
            .map_or(false, |it| it.kind() == T!(,))
    }

    pub fn has_multiline_values(&self, toml_version: TomlVersion) -> bool {
        self.key_values().any(|key_value| {
            key_value.value().map_or(false, |value| match value {
                crate::Value::Array(array) => array.should_be_multiline(toml_version),
                crate::Value::InlineTable(inline_table) => {
                    inline_table.should_be_multiline(toml_version)
                }
                crate::Value::MultiLineBasicString(string) => {
                    string.token().unwrap().text().contains('\n')
                }
                crate::Value::MultiLineLiteralString(string) => {
                    string.token().unwrap().text().contains('\n')
                }
                _ => false,
            })
        })
    }

    pub fn has_inner_comments(&self) -> bool {
        support::has_inner_comments(self.syntax().children_with_tokens(), T!('{'), T!('}'))
    }
}

impl Key {
    pub fn token(&self) -> Option<syntax::SyntaxToken> {
        match self {
            Key::BareKey(key) => key.token(),
            Key::BasicString(key) => key.token(),
            Key::LiteralString(key) => key.token(),
        }
    }
}
