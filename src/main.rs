use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use cargo_util::ProcessBuilder;
use clap::Parser;
use tempfile::NamedTempFile;
use tracing::{debug, error, info};

use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use ulid::Ulid;

pub mod settings;

use self::settings::{CompiledProfile, Settings};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file with profiles to run
    #[arg(short, long)]
    config: PathBuf,

    /// Path to directory where to put compiled mesh files
    #[arg(short, long)]
    out: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let settings: Settings = {
        let file = File::open(args.config)?;
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader)?;
        u
    };

    let profiles = settings.compile_profiles();
    let total_configurations: usize = profiles.values().into_iter().map(HashSet::len).sum();

    info!(
        profiles = profiles.keys().len(),
        total_configurations = total_configurations,
        "Settings compiled"
    );

    let profiles_flatten: Vec<_> = profiles.values().flatten().collect();
    let style = ProgressStyle::default_bar();

    let _results: Vec<anyhow::Result<()>> = profiles_flatten
        .par_iter()
        .map(|profile| compile_stl_from_profile(profile, args.out.as_path()))
        .progress_with_style(style)
        .collect();
    Ok(())
}

fn compile_stl_from_profile(profile: &CompiledProfile, out_dir: &Path) -> anyhow::Result<()> {
    let mut script = NamedTempFile::new_in("./")?;
    let id = Ulid::new();
    info!("wat {}", id.to_string());
    let outfile = out_dir
        .to_owned()
        .join(profile.name())
        .join(format!("{}.3mf", id.to_string()));

    info!(profile = ?profile, outfile = %outfile.display(), "Processing...");

    profile.write_script(&mut script)?;
    script.flush()?;

    let script = script; // downgrade to readonly

    debug!(script = %script.path().display(), params = ?profile.params(), "Wrote scipt file");

    let out_dir = outfile.parent().unwrap();
    create_dir_all(out_dir)?;

    let cmd = {
        let mut cmd = ProcessBuilder::new("openscad");
        cmd.args(&["--export-format", "3mf"])
            .args(&["-o", outfile.display().to_string().as_str()])
            .arg(script.path());
        cmd
    };
    let result = cmd.exec_with_streaming(
        &mut make_debug_logger(&profile),
        &mut make_debug_logger(&profile),
        false,
    )?;

    if result.status.success() {
        Ok(())
    } else {
        anyhow::bail!("Ooopsie")
    }
}

#[allow(dead_code)]
fn make_error_logger(profile: &CompiledProfile) -> impl FnMut(&str) -> anyhow::Result<()> {
    let name = profile.name().to_owned();
    let params = profile.params().to_owned();
    move |msg| {
        error!(profile_name = name, params = ?params, msg);
        Ok(())
    }
}

fn make_debug_logger(profile: &CompiledProfile) -> impl FnMut(&str) -> anyhow::Result<()> {
    let name = profile.name().to_owned();
    let params = profile.params().to_owned();
    move |msg| {
        debug!(profile_name = name, params = ?params, msg);
        Ok(())
    }
}

// openscad  --export-format 3mf -o test.3mf asd
