use std::env;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::process;
use zx0::Compressor;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage(program_name: String) -> ! {
    eprintln!("Usage: {} [OPTIONS] INPUT [OUTPUT]", program_name.rsplit('/').next().unwrap());
    eprintln!();
    eprintln!("Options:");
    eprintln!("    -h, --help         Display this message");
    eprintln!("    -V, --version      Print version info and exit");
    eprintln!("    -f, --force        Force overwrite of output file");
    eprintln!("    -c, --classic      Classic file format (v1.*)");
    eprintln!("    -b, --backwards    Compress backwards");
    eprintln!("    -q, --quick        Quick non-optimal compression");
    eprintln!("    -Q, --quiet        Do not show any progress or summary information");
    eprintln!("    -s, --skip AMOUNT  Skip AMOUNT bytes of input data");

    process::exit(1);
}

fn version() -> ! {
    eprintln!("zx0-rs {}\nBased on ZX0 v2.2 by Einar Saukas", VERSION);
    process::exit(1);
}

fn main() {
    let mut compressor = Compressor::new();

    let mut input_filename = None;
    let mut output_filename = None;

    let mut backwards_mode = false;
    let mut forced_mode = false;
    let mut quiet_mode = false;

    let mut skip = 0;

    let mut iter = env::args();
    let program_name = iter.next().unwrap_or_else(|| {
        eprintln!("error: expected at least one argument containing the program name");
        process::exit(1);
    });

    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "-c" | "--classic" => { compressor.classic_mode(true); },
            "-b" | "--backwards" => {
                backwards_mode = true;
                compressor.backwards_mode(true);
            },
            "-q" | "--quick" => { compressor.quick_mode(true); },
            "-f" | "--force" => { forced_mode = true; },
            "-Q" | "--quiet" => { quiet_mode = true; },
            "-h" | "--help" => usage(program_name),
            "-V" | "--version" => version(),
            "-s" | "--skip" => {
                if let Some(argument) = iter.next() {
                    if let Ok(value) = argument.parse() {
                        skip = value;
                        compressor.skip(value);
                    } else {
                        eprintln!("error: expected integer value for skip argument");
                        process::exit(1);
                    }
                } else {
                    eprintln!("error: expected value for skip argument");
                    process::exit(1);
                }
            }
            _ => {
                if argument.starts_with('-') {
                    eprintln!("error: unrecognized argument: {}", argument);
                    process::exit(1);
                } else if input_filename.is_none() {
                    input_filename = Some(argument);
                } else if output_filename.is_none() {
                    output_filename = Some(argument);
                } else {
                    eprintln!("error: too many filename arguments provided");
                    process::exit(1);
                }
            }
        }
    }

    // Unwrap and optionally generate filenames
    let input_filename = input_filename.unwrap_or_else(|| usage(program_name));
    let output_filename = output_filename.unwrap_or_else(|| format!("{}.zx0", input_filename));

    // Read input file
    let mut input = fs::read(&input_filename).unwrap_or_else(|err| {
        eprintln!("error: could not read input file: {}", err);
        process::exit(1);
    });

    // Validate skip length
    if skip >= input.len() {
        eprintln!("error: skipping entire input file");
        process::exit(1);
    }

    // Check if output file already exists
    if !forced_mode {
        match File::open(&output_filename) {
            Ok(_) => {
                eprintln!("error: output file already exists and --force was not specified");
                process::exit(1);
            },
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => {
                eprintln!("error: could not open output file: {}", err);
                process::exit(1);
            }
        };
    }

    // Reverse the input if working backwards
    if backwards_mode {
        input.reverse();
    }

    if !quiet_mode {
        compressor.progress_callback(|progress| {
            print!("\rProgress: {:.1} %", progress * 100.0);

            if let Err(err) = std::io::stdout().flush() {
                eprintln!("error: could not flush stdout: {}", err);
                process::exit(1);
            }
        });
    }

    // Compress
    let mut result = compressor.compress(&input);

    // Reverse the output if working backwards
    if backwards_mode {
        result.output.reverse();
    }

    // Write output file
    if let Err(err) = fs::write(&output_filename, &result.output) {
        eprintln!("error: could not write to output file: {}", err);
        process::exit(1);
    }

    // Print a summary
    if !quiet_mode {
        println!(
            "\r{} ({} bytes) -> {} ({} bytes), ratio = {:.3}, delta = {}",
            input_filename,
            input.len(),
            output_filename,
            result.output.len(),
            input.len() as f32 / result.output.len() as f32,
            result.delta
        );
    }
}
