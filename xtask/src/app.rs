use crate::command;
use clap::Parser;

#[derive(clap::Parser)]
#[command(name = "toml", version)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: command::XTaskCommand,
}

impl<I, T> From<I> for Args
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    fn from(value: I) -> Self {
        Self::parse_from(value)
    }
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args = args.into();
    match args.subcommand {
        command::XTaskCommand::Codegen(subcommand) => match subcommand {
            command::CodeGenCommand::Grammer(args) => command::codegen_grammer::run(args)?,
        },
    }
    Ok(())
}
