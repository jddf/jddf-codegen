use crate::target;
use clap::{App, Arg, ArgMatches};
use failure::format_err;
use failure::Error;
use inflector::Inflector;
use jddf::{Form, Schema, Type};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub struct Target {
    out_path: PathBuf,
    root_name: String,
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

            // Infer a root name from the file name of the input schema.
            let input = PathBuf::from(matches.value_of("INPUT").unwrap());

            let input_file_name = input
                .file_name()
                .ok_or(format_err!("Could not infer file name from INPUT"))?;

            let input_file_name = input_file_name
                .to_str()
                .ok_or(format_err!("Could not convert INPUT file name to UTF-8"))?;

            let root_name = input_file_name.split(".").next().unwrap().to_pascal_case();

            Ok(Some(Target {
                out_path,
                root_name,
            }))
        } else {
            Ok(None)
        }
    }

    fn transform(&self, schema: &Schema) -> Result<Ast, Error> {
        let mut seq = vec![];

        if let Some(defs) = schema.definitions() {
            for (name, schema) in defs {
                let ast = self.transform_subschema(&mut seq, &mut vec![name], schema)?;
                self.ensure_has_name(Some(name), &mut seq, ast);
            }
        }

        let ast = self.transform_subschema(&mut seq, &mut vec![&self.root_name], schema)?;
        self.ensure_has_name(None, &mut seq, ast);

        Ok(Ast::Sequence(seq))
    }

    fn serialize(&self, ast: &Ast) -> Result<(), Error> {
        let mut out = BufWriter::new(File::create(self.out_path.clone())?);
        self.serialize_ast(&mut out, ast)?;

        Ok(())
    }
}

impl Target {
    fn transform_subschema<'a>(
        &self,
        seq: &mut Vec<Ast>,
        name: &mut Vec<&'a str>,
        schema: &'a Schema,
    ) -> Result<Ast, Error> {
        match schema.form() {
            Form::Empty => Ok(Ast::Any),
            Form::Ref(def) => Ok(Ast::Identifier(self.name(&[&def]))),
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
            Form::Enum(vals) => Ok(Ast::Union(
                vals.iter()
                    .map(|val| Ast::Constant(val.to_owned()))
                    .collect(),
            )),
            Form::Elements(schema) => Ok(Ast::Array(Box::new(
                self.transform_subschema(seq, name, schema)?,
            ))),
            Form::Properties {
                required, optional, ..
            } => {
                let mut props = Vec::new();
                for (prop, schema) in required {
                    name.push(prop);
                    let ast = self.transform_subschema(seq, name, schema)?;
                    name.pop();

                    props.push((prop.to_owned(), true, ast));
                }

                for (prop, schema) in optional {
                    name.push(prop);
                    let ast = self.transform_subschema(seq, name, schema)?;
                    name.pop();

                    props.push((prop.to_owned(), false, ast));
                }

                let id = self.name(name);
                seq.push(Ast::Interface(id.clone(), props));
                Ok(Ast::Identifier(id))
            }
            Form::Discriminator(tag, mapping) => {
                let mut cases = Vec::new();
                for (val, schema) in mapping {
                    name.push(val);

                    // We know that `schema` is of the properties form. We want
                    // to do almost exactly the same thing as we did for the
                    // properties form, but with one additional property, for
                    // the discriminator tag.
                    if let Form::Properties {
                        required, optional, ..
                    } = schema.form()
                    {
                        let mut props = Vec::new();
                        props.push((tag.to_owned(), true, Ast::Constant(val.to_owned())));

                        for (prop, schema) in required {
                            name.push(prop);
                            let ast = self.transform_subschema(seq, name, schema)?;
                            name.pop();

                            props.push((prop.to_owned(), true, ast));
                        }

                        for (prop, schema) in optional {
                            name.push(prop);
                            let ast = self.transform_subschema(seq, name, schema)?;
                            name.pop();

                            props.push((prop.to_owned(), false, ast));
                        }

                        let id = self.name(name);
                        seq.push(Ast::Interface(id.clone(), props));
                        cases.push(Ast::Identifier(id));
                    }

                    name.pop();
                }

                Ok(Ast::Union(cases))
            }
            Form::Values(schema) => Ok(Ast::Map(Box::new(
                self.transform_subschema(seq, name, schema)?,
            ))),
        }
    }

    fn ensure_has_name(&self, name: Option<&str>, seq: &mut Vec<Ast>, ast: Ast) {
        match ast {
            Ast::Identifier(_) => {}
            _ => {
                let id = self.name(&[name.unwrap_or(&self.root_name)]);
                seq.push(Ast::Typedef(id, Box::new(ast)));
            }
        }
    }

    fn serialize_ast(&self, w: &mut Write, ast: &Ast) -> Result<(), Error> {
        match ast {
            Ast::Any => write!(w, "any")?,
            Ast::Boolean => write!(w, "boolean")?,
            Ast::Number => write!(w, "number")?,
            Ast::String => write!(w, "string")?,
            Ast::Constant(s) => write!(w, "{:?}", s)?,
            Ast::Array(ast) => {
                self.serialize_ast(w, ast)?;
                write!(w, "[]")?;
            }
            Ast::Map(ast) => {
                write!(w, "{{ [name: string]: ")?;
                self.serialize_ast(w, ast)?;
                write!(w, "}}")?;
            }
            Ast::Interface(name, props) => {
                writeln!(w, "export interface {} {{", name)?;
                for (name, required, ast) in props {
                    write!(w, "  {}{}: ", name, if *required { "" } else { "?" })?;
                    self.serialize_ast(w, ast)?;
                    writeln!(w, ";")?;
                }
                writeln!(w, "}}")?;
            }
            Ast::Union(asts) => {
                for ast in &asts[..asts.len() - 1] {
                    self.serialize_ast(w, ast)?;
                    write!(w, " | ")?;
                }
                self.serialize_ast(w, &asts[asts.len() - 1])?;
            }
            Ast::Identifier(id) => write!(w, "{}", id)?,
            Ast::Typedef(name, ast) => {
                write!(w, "export type {} = ", name)?;
                self.serialize_ast(w, ast)?;
                writeln!(w, ";")?;
            }
            Ast::Sequence(asts) => {
                for ast in asts {
                    self.serialize_ast(w, ast)?;
                    writeln!(w)?;
                }
            }
        }

        Ok(())
    }

    fn name(&self, name: &[&str]) -> String {
        name.join("_").to_pascal_case()
    }
}

#[derive(Debug)]
pub enum Ast {
    Any,
    Boolean,
    Number,
    String,
    Constant(String),
    Array(Box<Ast>),
    Map(Box<Ast>),
    Interface(String, Vec<(String, bool, Ast)>),
    Union(Vec<Ast>),
    Identifier(String),
    Typedef(String, Box<Ast>),
    Sequence(Vec<Ast>),
}
