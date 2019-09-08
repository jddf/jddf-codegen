mod target;
mod targets;

use clap::{App, AppSettings, Arg};
use target::Target;

fn main() {
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
    println!("{:?}", matches);
}
