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
    pkg_name: String,
}

impl target::Target for Target {
    type Ast = Ast;

    fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("go-out")
                .help("Golang output directory")
                .takes_value(true)
                .long("go-out"),
        )
    }

    fn from_args(matches: &ArgMatches) -> Result<Option<Self>, Error> {
        if let Some(go_out) = matches.value_of("go-out") {
            // Infer a package name from the output directory.
            let pkg_name = PathBuf::from(go_out)
                .components()
                .last()
                .ok_or(format_err!(
                    "Could not determine package name from --go-out"
                ))?
                .as_os_str()
                .to_str()
                .ok_or(format_err!("Could not convert --go-out to UTF-8"))?
                .to_snake_case();

            // Infer a root name from the file name of the input schema.
            let input = PathBuf::from(matches.value_of("INPUT").unwrap());

            let input_file_name = input
                .file_name()
                .ok_or(format_err!("Could not infer file name from INPUT"))?;

            let input_file_name = input_file_name
                .to_str()
                .ok_or(format_err!("Could not convert INPUT file name to UTF-8"))?;

            let root_name = input_file_name.split(".").next().unwrap().to_snake_case();

            let out_path = PathBuf::from(go_out).join(format!("{}.go", root_name));

            Ok(Some(Target {
                out_path,
                root_name,
                pkg_name,
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

        writeln!(out, "package {}", self.pkg_name)?;
        writeln!(out, "import \"time\"")?;
        writeln!(out, "import \"encoding/json\"")?;
        writeln!(out, "import \"errors\"")?;
        writeln!(
            out,
            "var ErrUnknownVariant = errors.New(\"{}: unknown discriminator tag value\")",
            self.pkg_name
        )?;
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
            Form::Empty => Ok(Ast::EmptyInterface),
            Form::Ref(def) => Ok(Ast::Identifier(self.name(&[&def]))),
            Form::Type(Type::Boolean) => Ok(Ast::Boolean),
            Form::Type(Type::String) => Ok(Ast::String),
            Form::Type(Type::Timestamp) => Ok(Ast::Time),
            Form::Type(Type::Int8) => Ok(Ast::Int8),
            Form::Type(Type::Int16) => Ok(Ast::Int16),
            Form::Type(Type::Int32) => Ok(Ast::Int32),
            Form::Type(Type::Uint8) => Ok(Ast::Uint8),
            Form::Type(Type::Uint16) => Ok(Ast::Uint16),
            Form::Type(Type::Uint32) => Ok(Ast::Uint32),
            Form::Type(Type::Float32) => Ok(Ast::Float32),
            Form::Type(Type::Float64) => Ok(Ast::Float64),
            Form::Enum(vals) => {
                let enum_name = self.name(name);
                seq.push(Ast::Typedef(enum_name.clone(), Box::new(Ast::String)));

                for val in vals {
                    name.push(val);
                    seq.push(Ast::Const(
                        self.name(name),
                        enum_name.clone(),
                        Box::new(Ast::StrConstant(val.to_owned())),
                    ));
                    name.pop();
                }

                Ok(Ast::Identifier(enum_name))
            }
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

                    props.push(Property {
                        name: prop.to_pascal_case(),
                        required: true,
                        json: prop.to_owned(),
                        ast,
                    })
                }

                for (prop, schema) in optional {
                    name.push(prop);
                    let ast = self.transform_subschema(seq, name, schema)?;
                    name.pop();

                    props.push(Property {
                        name: prop.to_pascal_case(),
                        required: false,
                        json: prop.to_owned(),
                        ast,
                    });
                }

                let id = self.name(name);
                seq.push(Ast::Struct(id.clone(), props));
                Ok(Ast::Identifier(id))
            }
            Form::Discriminator(tag, mapping) => {
                // Create the enum for the values the tag can take on.
                name.push(tag);
                let tag_enum_name = self.name(name);
                seq.push(Ast::Typedef(tag_enum_name.clone(), Box::new(Ast::String)));
                name.pop();

                // Loop over the mapping values. For each one, we must generate
                // both a value for the tag enum, and a variant struct.
                let mut variants = Vec::new();
                for (tag_value, variant) in mapping {
                    // Add a value for the tag enum.
                    name.push(tag);
                    name.push(tag_value);
                    let tag_variant_name = self.name(name);
                    seq.push(Ast::Const(
                        tag_variant_name,
                        tag_enum_name.clone(),
                        Box::new(Ast::StrConstant(tag_value.to_owned())),
                    ));
                    name.pop();
                    name.pop();

                    // Generate a variant struct. We can count on the variant
                    // being of the properties.
                    if let Form::Properties {
                        required, optional, ..
                    } = variant.form()
                    {
                        let mut props = Vec::new();
                        for (prop, schema) in required {
                            name.push(prop);
                            let ast = self.transform_subschema(seq, name, schema)?;
                            name.pop();

                            props.push(Property {
                                name: prop.to_pascal_case(),
                                required: true,
                                json: prop.to_owned(),
                                ast,
                            })
                        }

                        for (prop, schema) in optional {
                            name.push(prop);
                            let ast = self.transform_subschema(seq, name, schema)?;
                            name.pop();

                            props.push(Property {
                                name: prop.to_pascal_case(),
                                required: false,
                                json: prop.to_owned(),
                                ast,
                            });
                        }

                        name.push(tag_value);
                        let variant_name = self.name(name);
                        variants.push(DiscriminatorVariant {
                            name: variant_name,
                            name_json: tag_value.to_owned(),
                            properties: props,
                        });
                    }

                    name.pop();
                }

                let id = self.name(name);
                seq.push(Ast::DiscriminatorStruct {
                    name: id.clone(),
                    tag: tag_enum_name,
                    tag_short: tag.to_pascal_case(),
                    tag_json: tag.to_owned(),
                    variants,
                });
                Ok(Ast::Identifier(id))
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
            Ast::EmptyInterface => write!(w, "interface{{}}")?,
            Ast::Boolean => write!(w, "bool")?,
            Ast::Int8 => write!(w, "int8")?,
            Ast::Uint8 => write!(w, "uint8")?,
            Ast::Int16 => write!(w, "int16")?,
            Ast::Uint16 => write!(w, "uint16")?,
            Ast::Int32 => write!(w, "int32")?,
            Ast::Uint32 => write!(w, "uint32")?,
            Ast::Float32 => write!(w, "float32")?,
            Ast::Float64 => write!(w, "float64")?,
            Ast::String => write!(w, "string")?,
            Ast::Time => write!(w, "time.Time")?,
            Ast::StrConstant(s) => write!(w, "{:?}", s)?,
            Ast::Const(name, ty, ast) => {
                write!(w, "const {} {} = ", name, ty)?;
                self.serialize_ast(w, ast)?;
                writeln!(w)?;
            }
            Ast::Array(ast) => {
                write!(w, "[]")?;
                self.serialize_ast(w, ast)?;
            }
            Ast::Map(ast) => {
                write!(w, "map[string]")?;
                self.serialize_ast(w, ast)?;
            }
            Ast::Struct(name, props) => {
                writeln!(w, "type {} struct {{", name)?;
                for prop in props {
                    let Property {
                        name,
                        required,
                        json,
                        ast,
                    } = prop;

                    write!(w, "\t{} {}", name, if *required { "" } else { "*" })?;
                    self.serialize_ast(w, ast)?;
                    writeln!(w, " `json:\"{}\"`", json)?;
                }
                writeln!(w, "}}")?;
            }
            Ast::DiscriminatorStruct {
                name,
                tag_short,
                tag,
                tag_json,
                variants,
            } => {
                writeln!(w, "type {} struct {{", name)?;
                writeln!(w, "\t{} {} `json:{:?}`", tag_short, tag, tag_json)?;
                for variant in variants {
                    writeln!(w, "\t{}", variant.name)?;
                }
                writeln!(w, "}}")?;
                writeln!(w)?;

                writeln!(w, "func (v {}) MarshalJSON() ([]byte, error) {{", name)?;
                writeln!(w, "\tswitch v.{} {{", tag_short)?;
                for variant in variants {
                    writeln!(w, "\tcase {:?}:", variant.name_json)?;
                    writeln!(w, "\t\treturn json.Marshal(struct {{ Tag string `json:{:?}`; {} }}{{ Tag: {:?}, {}: v.{} }});", tag_json, variant.name, variant.name_json, variant.name, variant.name)?;
                }
                writeln!(w, "\t}}")?;
                writeln!(w, "\treturn nil, ErrUnknownVariant")?;
                writeln!(w, "}}")?;

                writeln!(w, "func (v *{}) UnmarshalJSON(b []byte) error {{", name)?;
                writeln!(w, "\tvar obj map[string]interface{{}}")?;
                writeln!(
                    w,
                    "\tif err := json.Unmarshal(b, &obj); err != nil {{ return err }}"
                )?;
                writeln!(w, "\ttag, ok := obj[{:?}].(string)", tag_json)?;
                writeln!(w, "\tif !ok {{ return ErrUnknownVariant }}")?;
                writeln!(w, "\tv.{} = tag", tag_short)?;
                writeln!(w, "\tswitch tag {{")?;
                for variant in variants {
                    writeln!(w, "\tcase {:?}:", variant.name_json)?;
                    writeln!(w, "\t\treturn json.Unmarshal(b, &v.{})", variant.name)?;
                }
                writeln!(w, "\t}}")?;
                writeln!(w, "\treturn ErrUnknownVariant")?;
                writeln!(w, "}}")?;

                for variant in variants {
                    writeln!(w, "type {} struct {{", variant.name)?;
                    for prop in &variant.properties {
                        let Property {
                            name,
                            required,
                            json,
                            ast,
                        } = prop;

                        write!(w, "\t{} {}", name, if *required { "" } else { "*" })?;
                        self.serialize_ast(w, ast)?;
                        writeln!(w, " `json:\"{}\"`", json)?;
                    }
                    writeln!(w, "}}")?;
                }
            }
            Ast::Identifier(id) => write!(w, "{}", id)?,
            Ast::Typedef(name, ast) => {
                write!(w, "type {} = ", name)?;
                self.serialize_ast(w, ast)?;
                writeln!(w)?;
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
    EmptyInterface,
    Boolean,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
    String,
    Time,
    StrConstant(String),
    Const(String, String, Box<Ast>),
    Array(Box<Ast>),
    Map(Box<Ast>),
    Struct(String, Vec<Property>),
    DiscriminatorStruct {
        // the name of the struct
        name: String,
        // the name of the tag as it appears in Golang
        tag: String,
        // the short name of the tag as it appears in Golang struct member names
        tag_short: String,
        // the name of the tag as it appears in JSON
        tag_json: String,
        // the mapping variants
        variants: Vec<DiscriminatorVariant>,
    },
    Identifier(String),
    Typedef(String, Box<Ast>),
    Sequence(Vec<Ast>),
}

#[derive(Debug)]
pub struct DiscriminatorVariant {
    // the variant's name as it appears in Golang
    name: String,
    // the variant's name as it appars in JSON
    name_json: String,
    // the properties of the variant
    properties: Vec<Property>,
}

#[derive(Debug)]
pub struct Property {
    name: String,
    required: bool,
    json: String,
    ast: Ast,
}
