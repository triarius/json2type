mod go;
mod parse;

use clap::Parser;
use color_eyre::eyre::{eyre, Result, WrapErr};
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name for the type
    #[arg(short, long)]
    name: String,

    /// File to read JSON, default: stdin
    #[arg(short, long)]
    input: Option<String>,

    /// File to output Go, default: stdout
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let reader: Box<dyn Read> = match args.input {
        None => Box::new(io::stdin()),
        Some(filename) => Box::new(
            File::open(&filename).wrap_err_with(|| format!("failed to open file: {filename}"))?,
        ),
    };

    let buffered_reader = io::BufReader::new(reader);
    let value = serde_json::from_reader(buffered_reader)?;
    let parsed = parse::parse_value(&args.name, value);

    if let parse::FieldType::Object(obj) = parsed {
        let go_struct = go::type_string(&obj);
        let mut writer: Box<dyn Write> = match args.output {
            None => Box::new(io::stdout()),
            Some(filename) => Box::new(File::create(filename)?),
        };
        Ok(writer.write_all(go_struct.as_bytes())?)
    } else {
        Err(eyre!("Top level must be an object"))
    }
}
