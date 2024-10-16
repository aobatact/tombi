use super::Format;

impl Format for ast::String {
    fn format<'a>(&self, _context: &'a crate::Context<'a>) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::AstNode;
    use rstest::rstest;

    #[rstest]
    #[case(r#"key = "value\""#)]
    #[case(r#"key    = "value\""#)]
    fn barestring_key_value(#[case] source: &str) {
        let p = parser::parse(source);
        let ast = ast::Root::cast(p.syntax_node()).unwrap();
        assert_eq!(ast.format_default(), r#"key = "value\""#);
    }
}
