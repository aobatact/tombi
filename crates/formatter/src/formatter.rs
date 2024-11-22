mod date_time_delimiter;
pub mod definitions;
mod line_ending;
pub mod options;

pub use date_time_delimiter::DateTimeDelimiter;
pub use line_ending::LineEnding;
use syntax::TomlVersion;

use std::{borrow::Cow, fmt::Write};

pub struct Formatter<'a> {
    version: TomlVersion,
    ident_depth: u8,
    defs: crate::Definitions,
    options: Cow<'a, crate::Options>,
    buf: &'a mut (dyn Write + 'a),
}

impl<'a> Formatter<'a> {
    #[inline]
    pub fn new(version: TomlVersion, buf: &'a mut (dyn Write + 'a)) -> Self {
        Self {
            version,
            ident_depth: 0,
            defs: Default::default(),
            options: Cow::Owned(crate::Options::default()),
            buf,
        }
    }

    #[inline]
    pub fn new_with_options(
        version: TomlVersion,
        buf: &'a mut (dyn Write + 'a),
        options: &'a crate::Options,
    ) -> Self {
        Self {
            version,
            ident_depth: 0,
            defs: Default::default(),
            options: Cow::Borrowed(options),
            buf,
        }
    }

    #[inline]
    pub fn version(&self) -> TomlVersion {
        self.version
    }

    #[inline]
    pub fn options(&self) -> &crate::Options {
        &self.options
    }

    #[inline]
    pub fn defs(&self) -> &crate::Definitions {
        &self.defs
    }

    #[inline]
    pub fn line_ending(&self) -> &'static str {
        match self.options.line_ending.unwrap_or_default() {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }

    #[inline]
    pub const fn date_time_delimiter(&self) -> &'static str {
        match self.defs.date_time_delimiter() {
            DateTimeDelimiter::T => "T",
            DateTimeDelimiter::Space => " ",
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.reset_ident();
    }

    #[inline]
    pub fn ident(&self) -> String {
        self.defs.ident(self.ident_depth)
    }

    #[inline]
    pub fn inc_ident(&mut self) {
        self.ident_depth += 1;
    }

    #[inline]
    pub fn dec_ident(&mut self) {
        self.ident_depth = self.ident_depth.saturating_sub(1);
    }

    #[inline]
    fn reset_ident(&mut self) {
        self.ident_depth = 0;
    }

    #[inline]
    pub fn with_reset_ident(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), std::fmt::Error>,
    ) -> Result<(), std::fmt::Error> {
        let depth = self.ident_depth;

        self.reset_ident();

        let result = f(self);

        self.ident_depth = depth;

        result
    }
}

impl std::fmt::Write for Formatter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s)
    }
}
