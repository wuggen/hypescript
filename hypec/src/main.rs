use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
}

impl Options {
    fn output_file(&self) -> PathBuf {
        if let Some(output_file) = &self.output_file {
            output_file.clone()
        } else {
            self.input_file.with_extension("hyc")
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::from_args();

    let mut input = String::new();
    File::open(&options.input_file)
        .map_err(|e| e.to_string())?
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;

    let ast = hypescript_lang::parse::parse(&input).map_err(|errs| {
        let mut err = String::new();
        for e in errs {
            writeln!(&mut err, "{e}").unwrap();
        }
        err
    })?;

    hypescript_lang::types::typecheck(&ast).map_err(|e| e.to_string())?;

    let instructions = hypescript_lang::codegen::translate(&ast).map_err(|e| e.to_string())?;

    let mut output = File::create(options.output_file()).map_err(|e| e.to_string())?;
    hypescript_bytecode::write_instructions(&mut output, &instructions)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
