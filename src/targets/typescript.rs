use crate::target;
use clap::{App, Arg, ArgMatches};
use failure::Error;
use jddf::{Form, Schema, Type};
use std::path::PathBuf;

pub struct Target {
    out_path: PathBuf,
}

impl target::Target for Target {
    type Ast = Ast;

    fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("ts-out")
                .help("Typescript output directory")
                .takes_value(true)
                .long("ts-out"),
        )
    }

    fn from_args(matches: &ArgMatches) -> Result<Option<Self>, Error> {
        if let Some(ts_out) = matches.value_of("ts-out") {
            let out_path = PathBuf::from(ts_out).join("index.ts");

            Ok(Some(Target { out_path }))
        } else {
            Ok(None)
        }
    }

    fn transform(&self, schema: &Schema) -> Result<Ast, Error> {
        self.transform_subschema(&[], schema)
    }

    fn serialize(&self, ast: &Ast) -> Result<(), Error> {
        Ok(())
    }
}

impl Target {
    fn transform_subschema(&self, name: &[&str], schema: &Schema) -> Result<Ast, Error> {
        match schema.form() {
            Form::Empty => Ok(Ast::Any),
            Form::Type(Type::Boolean) => Ok(Ast::Boolean),
            Form::Type(Type::String) | Form::Type(Type::Timestamp) => Ok(Ast::String),
            Form::Type(Type::Int8)
            | Form::Type(Type::Uint8)
            | Form::Type(Type::Int16)
            | Form::Type(Type::Uint16)
            | Form::Type(Type::Int32)
            | Form::Type(Type::Uint32)
            | Form::Type(Type::Float32)
            | Form::Type(Type::Float64) => Ok(Ast::Number),
        }
    }
}

pub enum Ast {
    Any,
    Boolean,
    Number,
    String,
    Identifier(String),
    Interface(Vec<(String, bool, Ast)>),
    Sequence(Vec<Ast>),
}
