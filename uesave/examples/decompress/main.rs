//! Decompress a (potentially compressed) save file to plain GVAS.
//!
//! Usage: cargo run --example decompress --features oodle -- <input.sav> <output.gvas>

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let input = args.next().ok_or("missing input path")?;
    let output = args.next().ok_or("missing output path")?;

    let mut reader = std::io::BufReader::new(std::fs::File::open(input)?);
    let data = uesave::compression::decompress_save(&mut reader)?;
    std::fs::write(output, data)?;
    Ok(())
}
