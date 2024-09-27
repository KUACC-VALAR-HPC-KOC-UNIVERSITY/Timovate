use clap::Parser;
use std::fs;
use std::io;
use timovate::{Cli, FileMover};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if !cli.source.is_dir() {
        eprintln!(
            "Error: Source '{}' is not a valid directory.",
            cli.source.display()
        );
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Invalid source directory",
        ));
    }

    // Check if the temporary directory exists
    if !cli.temporary.exists() {
        if cli.dry_run {
            println!(
                "[DRY RUN] Would create temporary directory '{}'",
                cli.temporary.display()
            );
        } else {
            // Attempt to create the temporary directory
            if let Err(e) = fs::create_dir_all(&cli.temporary) {
                eprintln!(
                    "Error: Could not create temporary directory '{}': {}",
                    cli.temporary.display(),
                    e
                );
                return Err(e);
            }

            if cli.verbose {
                println!("Created temporary directory '{}'", cli.temporary.display());
            }
        }
    } else if !cli.temporary.is_dir() {
        eprintln!(
            "Error: Temporary path '{}' exists but is not a directory.",
            cli.temporary.display()
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Temporary path is not a directory",
        ));
    }

    let mover = FileMover::new(&cli).map_err(|err| {
        eprintln!("Error initializing FileMover: {}", err);
        io::Error::new(io::ErrorKind::InvalidInput, err)
    })?;

    mover.execute()
}
