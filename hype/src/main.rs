use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use hypescript_vm::ExecutionContext;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    trace: bool,
    path: PathBuf,
}

fn main() {
    let Options { trace, path } = Options::from_args();

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Couldn't open {}: {}", path.display(), err);
            std::process::exit(1);
        }
    };

    let mut program = Vec::new();
    if let Err(err) = file.read_to_end(&mut program) {
        eprintln!("Error reading {}: {}", path.display(), err);
        std::process::exit(1);
    }

    let input_stream = BufReader::new(std::io::stdin());
    let output_stream = std::io::stdout();
    let context = ExecutionContext::new(&program)
        .with_input_stream(input_stream)
        .with_output_stream(output_stream);

    let context = if trace { context.with_trace() } else { context };

    match context.run() {
        Ok(summary) => {
            if trace {
                println!("{summary}");
            }
        }

        Err(err) => {
            eprintln!("Program halted with {}", err);
            std::process::exit(1);
        }
    }
}
