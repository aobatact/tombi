use crate::Lint;

impl Lint for ast::ArrayOfTables {
    fn lint(&self, l: &mut crate::Linter) {
        for key_value in self.key_values() {
            key_value.lint(l);
        }
    }
}
