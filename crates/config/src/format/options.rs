#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone)]
pub struct Options {
    /// The type of line ending.
    ///
    /// - `lf`: Line Feed only (`\n`), common on Linux and macOS as well as inside git repos.
    /// - `crlf`: Carriage Return Line Feed (`\r\n`), common on Windows.
    pub line_ending: Option<crate::format::LineEnding>,
}

impl Options {
    pub fn merge(&mut self, other: &Options) -> &mut Self {
        if let Some(line_ending) = other.line_ending {
            self.line_ending = Some(line_ending);
        }

        self
    }
}
