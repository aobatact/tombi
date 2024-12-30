use super::comment::{BeginDanglingComment, EndDanglingComment, LeadingComment, TailingComment};
use crate::Format;
use itertools::Itertools;
use std::fmt::Write;

impl Format for ast::Table {
    fn fmt(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let header = self.header().unwrap();

        for comment in self.header_leading_comments() {
            LeadingComment(comment).fmt(f)?;
        }

        write!(f, "[{header}]")?;

        if let Some(comment) = self.header_tailing_comment() {
            TailingComment(comment).fmt(f)?;
        }

        let key_values = self.key_values().collect_vec();

        if key_values.is_empty() {
            return Ok(());
        } else {
            write!(f, "{}", f.line_ending())?;

            self.begin_dangling_comments()
                .into_iter()
                .map(|comments| comments.into_iter().map(BeginDanglingComment).collect_vec())
                .collect_vec()
                .fmt(f)?;

            for (i, key_value) in key_values.into_iter().enumerate() {
                if i != 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                key_value.fmt(f)?;
            }

            for comments in self.end_dangling_comments() {
                comments
                    .into_iter()
                    .map(EndDanglingComment)
                    .collect_vec()
                    .fmt(f)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_format;

    test_format! {
        #[test]
        fn table_only_header(
            r#"[package]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_only_header_with_basic_string_key(
            r#"[dependencies."unicase"]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_only_header_nexted_keys(
            r#"[dependencies.unicase]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table(
            r#"
            [package]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_with_full_comment(
            r#"
            # header leading comment1
            # header leading comment2
            [header]  # header tailing comment
            # table begin dangling comment group 1-1
            # table begin dangling comment group 1-2

            # table begin dangling comment group 2-1
            # table begin dangling comment group 2-2
            # table begin dangling comment group 2-3

            # table begin dangling comment group 3-1

            # key value leading comment1
            # key value leading comment2
            key = "value"  # key tailing comment
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_begin_dangling_comment1(
            r#"
            [header]
            # table begin dangling comment group 1-1
            # table begin dangling comment group 1-2

            # table begin dangling comment group 2-1
            # table begin dangling comment group 2-2
            # table begin dangling comment group 2-3

            # table begin dangling comment group 3-1

            # key value leading comment1
            # key value leading comment2
            key = "value"  # key tailing comment
            "#
        ) -> Ok(source);
    }
}
