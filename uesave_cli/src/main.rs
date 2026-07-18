use std::fs::{self, File, OpenOptions};
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Cursor, Write};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use uesave::compression::CompressionFormat;
use uesave::games::palworld::{palworld_types, Palworld};
use uesave::{Game, NoGame, Save, SaveReader, StructType, Types};

#[derive(Parser, Debug)]
struct ActionToJson {
    #[arg(short, long, default_value = "-")]
    input: String,

    #[arg(short, long, default_value = "-")]
    output: String,

    /// Silence any parse warnings
    #[arg(long)]
    no_warn: bool,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,

    /// Enable Palworld custom property support
    #[arg(long)]
    palworld: bool,
}

#[derive(Parser, Debug)]
struct ActionFromJson {
    #[arg(short, long, default_value = "-")]
    input: String,

    #[arg(short, long, default_value = "-")]
    output: String,

    /// Enable Palworld custom property support (deserialize typed Pal structs)
    #[arg(long)]
    palworld: bool,

    /// Compress the output with Oodle (Palworld PLM format)
    #[arg(long)]
    compress_oodle: bool,
}

#[derive(Parser, Debug)]
struct ActionEdit {
    #[arg(required = true, index = 1)]
    path: String,

    /// Silence any parse warnings
    #[arg(long)]
    no_warn: bool,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,

    /// Enable Palworld custom property support
    #[arg(long)]
    palworld: bool,

    /// Compress the modified save with Oodle (Palworld PLM format)
    #[arg(long)]
    compress_oodle: bool,
}

#[derive(Parser, Debug)]
struct ActionTestResave {
    #[arg(required = true, index = 1)]
    path: String,

    /// If resave fails, write input.sav and output.sav to working directory for debugging
    #[arg(short, long)]
    debug: bool,

    /// Silence any parse warnings
    #[arg(long)]
    no_warn: bool,

    /// Trace and generate trace.json file
    #[cfg(feature = "tracing")]
    #[arg(long)]
    trace: bool,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,

    /// Enable Palworld custom property support
    #[arg(long)]
    palworld: bool,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Convert binary save to plain text JSON
    ToJson(ActionToJson),
    /// Convert JSON back to binary save
    FromJson(ActionFromJson),
    /// Launch editor to edit a save file as JSON in place
    Edit(ActionEdit),
    /// Test resave
    TestResave(ActionTestResave),
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

fn parse_type(t: &str) -> Result<(String, StructType)> {
    if let Some((l, r)) = t.rsplit_once('=') {
        Ok((l.to_owned(), r.into()))
    } else {
        Err(anyhow!("Malformed type"))
    }
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::ToJson(mut action) => {
            let types = build_types(action.palworld, std::mem::take(&mut action.r#type));
            if action.palworld {
                run_to_json::<Palworld>(action, types)?;
            } else {
                run_to_json::<NoGame>(action, types)?;
            }
        }
        Action::FromJson(io) => {
            if io.palworld {
                run_from_json::<Palworld>(io)?;
            } else {
                run_from_json::<NoGame>(io)?;
            }
        }
        Action::TestResave(mut action) => {
            let types = build_types(action.palworld, std::mem::take(&mut action.r#type));
            if action.palworld {
                run_test_resave::<Palworld>(action, types)?;
            } else {
                run_test_resave::<NoGame>(action, types)?;
            }
        }
        Action::Edit(mut action) => {
            let types = build_types(action.palworld, std::mem::take(&mut action.r#type));
            if action.palworld {
                run_edit::<Palworld>(action, types)?;
            } else {
                run_edit::<NoGame>(action, types)?;
            }
        }
    }
    Ok(())
}

/// Build the [`Types`] spec: the Palworld defaults when `palworld` is set, plus
/// any user-supplied `-t path=Type` overrides.
fn build_types(palworld: bool, overrides: Vec<(String, StructType)>) -> Types {
    let mut types = if palworld {
        palworld_types()
    } else {
        Types::new()
    };
    for (path, t) in overrides {
        types.add(path, t);
    }
    types
}

fn run_to_json<G: Game>(action: ActionToJson, types: Types) -> Result<()> {
    let save: Save<G> = SaveReader::new()
        .game::<G>()
        .log(!action.no_warn)
        .error_to_raw(true)
        .types(types)
        .read(input(&action.input)?)?;
    serde_json::to_writer_pretty(output(&action.output)?, &save)?;
    Ok(())
}

fn run_from_json<G: Game>(io: ActionFromJson) -> Result<()> {
    let save: Save<G> = serde_json::from_reader(&mut input(&io.input)?)?;
    if io.compress_oodle {
        save.write_compressed(&mut output(&io.output)?, CompressionFormat::Oodle)?;
    } else {
        save.write(&mut output(&io.output)?)?;
    }
    Ok(())
}

fn run_edit<G: Game>(action: ActionEdit, types: Types) -> Result<()> {
    let save: Save<G> = SaveReader::new()
        .game::<G>()
        .log(!action.no_warn)
        .error_to_raw(true)
        .types(types)
        .read(Cursor::new(fs::read(&action.path)?))?;
    let modified_save: Save<G> = serde_json::from_slice(&edit::edit_bytes_with_builder(
        serde_json::to_vec_pretty(&save)?,
        tempfile::Builder::new().suffix(".json"),
    )?)?;

    if save == modified_save {
        println!("File unchanged, doing nothing.");
    } else {
        println!("File modified, writing new save.");
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(action.path)?,
        );
        if action.compress_oodle {
            modified_save.write_compressed(&mut writer, CompressionFormat::Oodle)?;
        } else {
            modified_save.write(&mut writer)?;
        }
    }
    Ok(())
}

fn run_test_resave<G: Game>(action: ActionTestResave, mut types: Types) -> Result<()> {
    let path = std::path::Path::new(&action.path);

    if let Ok(types_file) = fs::read_to_string(path.with_extension("types")) {
        for t in types_file.lines() {
            if let Ok((path, t)) = parse_type(t) {
                types.add(path, t);
            }
        }
    }

    let write_debug = |name: &str, data: &[u8]| -> Result<()> {
        if action.debug {
            fs::write(name, data)?;
        }
        Ok(())
    };

    let input = fs::read(path)?;
    write_debug("input.sav", &input)?;

    let sr = SaveReader::new()
        .game::<G>()
        .log(!action.no_warn)
        .error_to_raw(true)
        .types(types);
    let mut reader = Cursor::new(&input);
    #[cfg(feature = "tracing")]
    let save: Save<G> = if action.trace {
        ser_hex::read("trace.json", &mut reader, |r| sr.read(r))?
    } else {
        sr.read(&mut reader)?
    };
    #[cfg(not(feature = "tracing"))]
    let save: Save<G> = sr.read(&mut reader)?;

    let mut output = vec![];
    save.write(&mut output)?;
    write_debug("output.sav", &output)?;
    if input != output {
        return Err(anyhow!("Resave did not match"));
    }

    let input_json = serde_json::to_vec_pretty(&save)?;
    write_debug("input.json", &input_json)?;

    let save_from_json: Save<G> = serde_json::from_slice(&input_json)?;
    let output_json = serde_json::to_vec_pretty(&save_from_json)?;
    write_debug("output.json", &output_json)?;

    let mut output = vec![];
    save_from_json.write(&mut output)?;
    write_debug("output.sav", &output)?;
    if input != output {
        return Err(anyhow!("JSON round trip did not match"));
    }
    println!("Resave successful");
    Ok(())
}

fn input<'a>(path: &str) -> Result<Box<dyn BufRead + 'a>> {
    Ok(match path {
        "-" => Box::new(BufReader::new(stdin().lock())),
        p => Box::new(BufReader::new(File::open(p)?)),
    })
}

fn output<'a>(path: &str) -> Result<Box<dyn Write + 'a>> {
    Ok(match path {
        "-" => Box::new(BufWriter::new(stdout().lock())),
        p => Box::new(BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(p)?,
        )),
    })
}
