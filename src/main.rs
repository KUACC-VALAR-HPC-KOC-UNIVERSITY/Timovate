use clap::Parser;
use std::fs;
use std::io;
use timovate::{Cli, FileMover, OperationMode};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.mode {
        OperationMode::Move => {
            // Ensure the source directory exists
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

            // Ensure the temporary directory exists or create it
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
        }
        OperationMode::Restore => {
            // Ensure the temporary directory exists (we are restoring from it)
            if !cli.temporary.is_dir() {
                eprintln!(
                    "Error: Temporary directory '{}' is not a valid directory.",
                    cli.temporary.display()
                );
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid temporary directory",
                ));
            }

            // Ensure the source directory exists or create it
            if !cli.source.exists() {
                if cli.dry_run {
                    println!(
                        "[DRY RUN] Would create source directory '{}'",
                        cli.source.display()
                    );
                } else {
                    // Attempt to create the source directory
                    if let Err(e) = fs::create_dir_all(&cli.source) {
                        eprintln!(
                            "Error: Could not create source directory '{}': {}",
                            cli.source.display(),
                            e
                        );
                        return Err(e);
                    }

                    if cli.verbose {
                        println!("Created source directory '{}'", cli.source.display());
                    }
                }
            } else if !cli.source.is_dir() {
                eprintln!(
                    "Error: Source path '{}' exists but is not a directory.",
                    cli.source.display()
                );
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Source path is not a directory",
                ));
            }
        }
    }

    let mover = FileMover::new(&cli).map_err(|err| {
        eprintln!("Error initializing FileMover: {}", err);
        io::Error::new(io::ErrorKind::InvalidInput, err)
    })?;

    mover.execute()
}
