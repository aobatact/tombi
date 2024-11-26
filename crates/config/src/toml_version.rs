/// TOML version.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum TomlVersion {
    /// TOML 1.0.0
    V1_0_0,

    #[default]
    /// TOML 1.1.0-preview
    V1_1_0_Preview,
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for TomlVersion {
    fn value_variants<'a>() -> &'a [Self] {
        use TomlVersion::*;

        &[V1_0_0, V1_1_0_Preview]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        use TomlVersion::*;

        Some(match self {
            V1_0_0 => clap::builder::PossibleValue::new("v1.0.0").help("TOML v1.0.0"),
            V1_1_0_Preview => {
                clap::builder::PossibleValue::new("v1.1.0-preview").help("TOML v1.1.0-preview")
            }
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn toml_version_comp() {
        assert!(crate::TomlVersion::V1_0_0 < crate::TomlVersion::V1_1_0_Preview);
    }
}
