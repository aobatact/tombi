mod builder;
mod error;
mod generated;
mod language;
mod version;

pub use builder::SyntaxTreeBuilder;
pub use error::{Error, SyntaxError};
pub use generated::SyntaxKind;
pub use language::TomlLanguage;
pub use version::TomlVersion;

pub type SyntaxNode = rg_tree::RedNode<TomlLanguage>;
pub type SyntaxToken = rg_tree::RedToken<TomlLanguage>;
pub type SyntaxElement = rg_tree::RedElement<TomlLanguage>;
pub type SyntaxNodeChildren = rg_tree::RedNodeChildren<TomlLanguage>;
pub type SyntaxElementChildren = rg_tree::RedElementChildren<TomlLanguage>;
pub type PreorderWithTokens = rg_tree::PreorderWithTokens<TomlLanguage>;

#[cfg(test)]
mod test {
    #[test]
    fn toml_version_comp() {
        assert!(crate::TomlVersion::V1_0_0 < crate::TomlVersion::V1_1_0_Preview);
    }
}
