use clap::{App, ArgMatches};
use failure::Error;
use jddf::Schema;

pub trait Target
where
    Self: Sized,
{
    type Ast;

    fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b>;
    fn from_args(matches: &ArgMatches) -> Result<Option<Self>, Error>;
    fn transform(&self, schema: &Schema) -> Result<Self::Ast, Error>;
    fn serialize(&self, ast: &Self::Ast) -> Result<(), Error>;
}
