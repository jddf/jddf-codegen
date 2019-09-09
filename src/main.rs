mod target;
mod targets;

use clap::{App, AppSettings, Arg};

use failure::Error;
use jddf::{Schema, SerdeSchema};
use std::fs::File;
use target::Target;
fn main() -> Result<(), Error> {
    let app = App::new("jddf-codegen")
        .version("0.1")
        .about("Generates data structures from JDDF schemas")
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("INPUT")
                .help("Input JDDF schema file")
                .last(true)
                .required(true),
        );

    let app = targets::typescript::Target::args(app);

    let matches = app.get_matches();

    let target_ts = targets::typescript::Target::from_args(&matches)?;

    // Parse out the input schema, and ensure it is valid.
    let input = matches.value_of("INPUT").unwrap();
    let file = File::open(input)?;
    let serde_schema: SerdeSchema = serde_json::from_reader(file)?;
    let schema = Schema::from_serde(serde_schema)?;

    // Run each of the target transformation routines. If any fail, do not
    // generate code.
    let ast_ts = if let Some(ref t) = target_ts {
        Some(t.transform(&schema)?)
    } else {
        None
    };

    // Serialize each of the ASTs. At this point, only IO errors can cause
    // issues.
    if let Some(ref t) = target_ts {
        t.serialize(&ast_ts.unwrap())?;
    }

    Ok(())
}
