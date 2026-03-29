use std::collections::HashMap;
use std::io::Read;

use clap::Parser;
use delbin::{generate, to_hex_string, Value};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Delbin: Descriptive Language for Binary Object\nGenerates binary firmware headers from a DSL description."
)]
struct Args {
    /// DSL input file path. Use '-' to read from stdin.
    input: String,

    /// Write output to FILE instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Output format: 'hex' (uppercase hex string) or 'bin' (raw bytes)
    #[arg(long, default_value = "hex", value_name = "FORMAT")]
    format: String,

    /// Set environment variable (may be repeated)
    #[arg(long = "env", value_name = "KEY=VALUE", action = clap::ArgAction::Append)]
    env_vars: Vec<String>,

    /// Load section data from file (may be repeated)
    #[arg(long = "section", value_name = "NAME=FILE", action = clap::ArgAction::Append)]
    sections: Vec<String>,

    /// Print warnings to stderr
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Read DSL source
    let dsl = if args.input == "-" {
        let mut s = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut s) {
            eprintln!("Error reading stdin: {e}");
            std::process::exit(1);
        }
        s
    } else {
        match std::fs::read_to_string(&args.input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {e}", args.input);
                std::process::exit(1);
            }
        }
    };

    // Parse --env KEY=VALUE pairs
    let mut env: HashMap<String, Value> = HashMap::new();
    for kv in &args.env_vars {
        if let Some((k, v)) = kv.split_once('=') {
            let value = if let Ok(n) = v.parse::<u64>() {
                Value::U64(n)
            } else {
                Value::String(v.to_string())
            };
            env.insert(k.to_string(), value);
        } else {
            eprintln!("Warning: ignoring malformed --env value (expected KEY=VALUE): {kv}");
        }
    }

    // Parse --section NAME=FILE pairs
    let mut sections: HashMap<String, Vec<u8>> = HashMap::new();
    for nf in &args.sections {
        if let Some((name, path)) = nf.split_once('=') {
            match std::fs::read(path) {
                Ok(data) => {
                    sections.insert(name.to_string(), data);
                }
                Err(e) => {
                    eprintln!("Error reading section '{name}' from '{path}': {e}");
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Warning: ignoring malformed --section value (expected NAME=FILE): {nf}");
        }
    }

    // Generate
    let result = match generate(&dsl, &env, &sections) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Print warnings if verbose
    if args.verbose {
        for w in &result.warnings {
            eprintln!("[{:?}] {}", w.code, w.message);
        }
    }

    // Format and write output
    let output_bytes: Vec<u8> = match args.format.as_str() {
        "hex" => {
            let hex = to_hex_string(&result.data);
            format!("{hex}\n").into_bytes()
        }
        "bin" => result.data,
        other => {
            eprintln!("Unknown --format '{other}'. Use 'hex' or 'bin'.");
            std::process::exit(1);
        }
    };

    match &args.output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &output_bytes) {
                eprintln!("Error writing '{path}': {e}");
                std::process::exit(1);
            }
        }
        None => {
            use std::io::Write;
            if let Err(e) = std::io::stdout().write_all(&output_bytes) {
                eprintln!("Error writing to stdout: {e}");
                std::process::exit(1);
            }
        }
    }
}
