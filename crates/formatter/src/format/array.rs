use crate::Format;
use std::fmt::Write;

impl Format for ast::Array {
    fn fmt(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "[{}", f.defs().array_bracket_inner_space())?;

        let values = self.values().collect::<Vec<_>>();
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            value.fmt(f)?;
        }

        write!(f, "{}]", f.defs().array_bracket_inner_space())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::AstNode;
    use rstest::rstest;

    #[rstest]
    #[case(r#"integers = [ 1, 2, 3 ]"#, r#"integers = [1, 2, 3]"#)]
    #[case(
        r#"colors = [ "red", "yellow", "green" ]"#,
        r#"colors = ["red", "yellow", "green"]"#
    )]
    #[case(
        r#"nested_arrays_of_ints = [ [ 1, 2 ], [ 3, 4, 5 ] ]"#,
        r#"nested_arrays_of_ints = [[1, 2], [3, 4, 5]]"#
    )]
    #[case(
        r#"nested_mixed_array = [ [ 1, 2 ], [ "a", "b", "c" ] ]"#,
        r#"nested_mixed_array = [[1, 2], ["a", "b", "c"]]"#
    )]
    #[case(
        r#"string_array = [ "all", 'strings', """are the same""", '''type''' ]"#,
        r#"string_array = ["all", 'strings', """are the same""", '''type''']"#
    )]
    fn single_line_array(#[case] source: &str, #[case] expected: &str) {
        let p = parser::parse(source);
        let ast = ast::Root::cast(p.syntax_node()).unwrap();

        let mut formatted_text = String::new();
        ast.fmt(&mut crate::Formatter::new(&mut formatted_text))
            .unwrap();

        assert_eq!(formatted_text, expected);
        assert_eq!(p.errors(), []);
    }
}
