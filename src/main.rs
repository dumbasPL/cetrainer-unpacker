mod archive;
mod decrypt;
mod pe;

use anyhow::Result;
use archive::parse_and_decompress;
use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser)]
struct Opts {
    #[clap(required = true)]
    input: String,
}

fn main() -> Result<()> {
    let args = Opts::parse();

    let canonical_path = fs::canonicalize(&args.input)?;
    let parent_dir = canonical_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?;
    let filename = canonical_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?
        .to_string_lossy();

    let data = fs::read(&args.input)?;

    if filename.to_lowercase().ends_with(".cetrainer") {
        let filename: String = PathBuf::from(&args.input)
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?
            .to_string_lossy()
            .into();

        println!("Extracting CETRAINER file");
        let data = decrypt::decrypt_trainer(&data)?;
        let path = parent_dir.join(format!("{}_decrypted.CETRAINER", filename));
        fs::write(path, data)?;
        return Ok(());
    }

    let (tiny, archive_data) = pe::get_archive(&data)?;

    if tiny {
        println!("Extracting tiny trainer");
        let path = parent_dir.join(format!("{}.CETRAINER", filename));
        let data = decrypt::decrypt_trainer(&archive_data)?;
        fs::write(path, data)?;
        return Ok(());
    }

    let (_, files) = parse_and_decompress(archive_data)
        .map_err(|_| anyhow::anyhow!("Failed to parse and decompress archive"))?;

    if files.is_empty() {
        Err(anyhow::anyhow!("No files found in archive"))?;
    }

    let canonical_path = fs::canonicalize(&args.input)?;
    let output_dir = canonical_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(format!(
            "{}_extracted",
            canonical_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    println!("Extracting files to: {}", output_dir.display());
    fs::create_dir_all(&output_dir)?;

    for file in files {
        let path = format!("{}{}", file.folder, file.filename).replace("\\", "/"); // f u wi*dows
        println!("Extracting: {}", path);

        let mut data = file.data;

        if path.ends_with(".CETRAINER") {
            data = decrypt::decrypt_trainer(&data)?;
        }

        let output_path = output_dir.join(path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, &data)?;
    }

    Ok(())
}
