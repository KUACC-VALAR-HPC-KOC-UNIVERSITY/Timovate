pub use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use regex::Regex;
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Parser)]
#[command(
    name = "Timovate",
    about = "Moves files based on their modification time"
)]
pub struct Cli {
    /// Source directory to search for files / restore to
    #[arg(short, long)]
    pub source: PathBuf,

    /// Directory to move files to / restore from
    #[arg(short, long)]
    pub temporary: PathBuf,

    /// Time criteria for moving files (e.g., '+30', '-15', '0' days) similar to find's -mtime
    #[arg(long, allow_hyphen_values = true, default_value = "+30")]
    pub days: String,

    /// Perform a dry run without moving files
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose mode
    #[arg(short, long)]
    pub verbose: bool,

    /// Operation mode: move or restore
    #[arg(short, long, value_enum, default_value = "move")]
    pub mode: OperationMode,

    /// Regex pattern to exclude files or directories
    #[arg(short, long, num_args(1..))]
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, ValueEnum)]
pub enum OperationMode {
    Move,
    Restore,
}

#[derive(Debug)]
enum TimeComparison {
    Exact(u64),
    MoreThan(u64),
    LessThan(u64),
}

pub struct FileMover {
    source: PathBuf,
    temporary: PathBuf,
    time_comparison: TimeComparison,
    dry_run: bool,
    verbose: bool,
    mode: OperationMode,
    exclude_regex: Option<Vec<Regex>>,
    pub stats: Arc<FileStats>,
}

impl FileMover {
    pub fn new(cli: &Cli) -> Result<Self, String> {
        let time_comparison = Self::parse_time_comparison(&cli.days)?;

        // Map the patterns into a vector of Regex objects
        let exclude_regex = if let Some(patterns) = &cli.exclude {
            let regexes = patterns
                .iter()
                .map(|pattern| {
                    Regex::new(pattern)
                        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Some(regexes)
        } else {
            None
        };

        Ok(Self {
            source: cli.source.clone(),
            temporary: cli.temporary.clone(),
            time_comparison,
            dry_run: cli.dry_run,
            verbose: cli.verbose,
            mode: cli.mode.clone(),
            exclude_regex,
            stats: Arc::new(FileStats::default()),
        })
    }

    fn parse_time_comparison(input: &str) -> Result<TimeComparison, String> {
        if input.starts_with('+') {
            let days_str = &input[1..];
            let days = days_str
                .parse::<u64>()
                .map_err(|_| format!("Invalid days input: '{}'", input))?;
            Ok(TimeComparison::MoreThan(days))
        } else if input.starts_with('-') {
            let days_str = &input[1..];
            let days = days_str
                .parse::<u64>()
                .map_err(|_| format!("Invalid days input: '{}'", input))?;
            Ok(TimeComparison::LessThan(days))
        } else {
            let days = input
                .parse::<u64>()
                .map_err(|_| format!("Invalid days input: '{}'", input))?;
            Ok(TimeComparison::Exact(days))
        }
    }

    pub fn execute(&self) -> io::Result<()> {
        match self.mode {
            OperationMode::Move => self.process_files(&self.source, &self.temporary)?,
            OperationMode::Restore => self.process_files(&self.temporary, &self.source)?,
        }

        println!(
            "Processed {} files and {} directories. Total size: {}",
            self.stats.files_moved.load(Ordering::SeqCst),
            self.stats.dirs_moved.load(Ordering::SeqCst),
            human_readable_size(self.stats.total_size.load(Ordering::SeqCst))
        );

        Ok(())
    }

    fn process_files(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.bfs_and_process(from, to)
    }

    fn bfs_and_process(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut queue = self.initialize_queue(from)?;

        while !queue.is_empty() {
            let current_level = self.get_current_level(&mut queue);
            let results = self.process_current_level(current_level, to);

            for result in results {
                let children = result?;
                queue.extend(children);
            }
        }

        Ok(())
    }

    fn initialize_queue(&self, from: &Path) -> io::Result<VecDeque<(PathBuf, PathBuf)>> {
        let mut queue = VecDeque::new();
        match fs::read_dir(from) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let src_path = entry.path();
                    let file_name = entry.file_name();
                    let rel_path = PathBuf::from(file_name);
                    queue.push_back((src_path, rel_path));
                }
            }
            Err(e) => {
                eprintln!("Error reading directory {}: {}", from.display(), e);
            }
        }
        Ok(queue)
    }

    fn get_current_level(
        &self,
        queue: &mut VecDeque<(PathBuf, PathBuf)>,
    ) -> Vec<(PathBuf, PathBuf)> {
        let level_size = queue.len();
        let mut current_level = Vec::with_capacity(level_size);

        for _ in 0..level_size {
            if let Some((current_src, rel_path)) = queue.pop_front() {
                current_level.push((current_src, rel_path));
            }
        }

        current_level
    }

    fn process_current_level(
        &self,
        current_level: Vec<(PathBuf, PathBuf)>,
        to: &Path,
    ) -> Vec<io::Result<Vec<(PathBuf, PathBuf)>>> {
        current_level
            .into_par_iter()
            .map(|(current_src, rel_path)| self.process_node(&current_src, &rel_path, to))
            .collect()
    }

    fn process_node(
        &self,
        current_src: &Path,
        rel_path: &Path,
        to: &Path,
    ) -> io::Result<Vec<(PathBuf, PathBuf)>> {
        // Check if the file or directory matches any of the exclude regex patterns
        if let Some(ref regexes) = self.exclude_regex {
            if regexes
                .iter()
                .any(|regex| regex.is_match(current_src.to_str().unwrap_or_default()))
            {
                if self.verbose {
                    println!("Excluding {} due to matching regex", current_src.display());
                }
                return Ok(vec![]); // Skip this file or directory
            }
        }

        let metadata = match fs::symlink_metadata(current_src) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!(
                    "Error accessing metadata for {}: {}",
                    current_src.display(),
                    e
                );
                return Ok(vec![]);
            }
        };

        let file_type = metadata.file_type();

        // Skip symbolic links
        if file_type.is_symlink() {
            if self.verbose {
                println!("Skipping symbolic link: {}", current_src.display());
            }
            return Ok(vec![]);
        }

        if file_type.is_dir() {
            self.process_directory_node(current_src, rel_path, to)
        } else if file_type.is_file() {
            self.process_file_node(current_src, rel_path, to, &metadata)
        } else {
            // Other types are ignored
            if self.verbose {
                println!("Skipping special file: {}", current_src.display());
            }
            Ok(vec![])
        }
    }

    fn process_directory_node(
        &self,
        current_src: &Path,
        rel_path: &Path,
        to: &Path,
    ) -> io::Result<Vec<(PathBuf, PathBuf)>> {
        if self.is_directory_matching(current_src)? {
            // Move the directory as a whole
            let current_dest = to.join(rel_path);
            self.move_entry(current_src, &current_dest, true)?;
            // Return empty vector to prevent processing children
            Ok(vec![])
        } else {
            // Directory does not match; collect its contents for the next level
            let mut children = Vec::new();
            match fs::read_dir(current_src) {
                Ok(entries) => {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        let file_name = entry.file_name();
                        let child_rel_path = rel_path.join(file_name);
                        children.push((path, child_rel_path));
                    }
                }
                Err(e) => {
                    eprintln!("Error reading directory {}: {}", current_src.display(), e);
                }
            }
            Ok(children)
        }
    }

    fn process_file_node(
        &self,
        current_src: &Path,
        rel_path: &Path,
        to: &Path,
        metadata: &fs::Metadata,
    ) -> io::Result<Vec<(PathBuf, PathBuf)>> {
        if self.is_file_matching(metadata) {
            let current_dest = to.join(rel_path);
            // Move the file
            self.move_entry(current_src, &current_dest, false)?;
        }
        Ok(vec![]) // Files don't have children
    }

    fn is_file_matching(&self, metadata: &fs::Metadata) -> bool {
        let modified = match metadata.modified() {
            Ok(time) => time,
            Err(_) => return false,
        };

        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let age_days = age / (24 * 60 * 60);

        match self.time_comparison {
            TimeComparison::Exact(n) => age_days == n,
            TimeComparison::MoreThan(n) => age_days > n,
            TimeComparison::LessThan(n) => age_days < n,
        }
    }

    fn is_directory_matching(&self, dir: &Path) -> io::Result<bool> {
        // Check if the directory matches any of the exclude regex patterns
        if let Some(ref regexes) = self.exclude_regex {
            if regexes
                .iter()
                .any(|regex| regex.is_match(dir.to_str().unwrap_or_default()))
            {
                if self.verbose {
                    println!(
                        "Excluding directory {} due to matching regex",
                        dir.display()
                    );
                }
                return Ok(false); // Excluded directory should not be moved
            }
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error reading directory {}: {}", dir.display(), e);
                return Ok(false); // Treat as not matching to avoid moving
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading entry in {}: {}", dir.display(), e);
                    continue;
                }
            };

            let path = entry.path();

            // Check if the entry matches any of the exclude regex patterns
            if let Some(ref regexes) = self.exclude_regex {
                if regexes
                    .iter()
                    .any(|regex| regex.is_match(path.to_str().unwrap_or_default()))
                {
                    if self.verbose {
                        println!("Excluding {} due to matching regex", path.display());
                    }
                    return Ok(false); // Cannot move directory if it contains excluded files
                }
            }

            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!("Error accessing metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            let file_type = metadata.file_type();

            if file_type.is_symlink() {
                // Skip symbolic links
                continue;
            }

            if file_type.is_dir() {
                if !self.is_directory_matching(&path)? {
                    return Ok(false);
                }
            } else if file_type.is_file() {
                if !self.is_file_matching(&metadata) {
                    return Ok(false);
                }
            } else {
                // Skip other special file types
                if self.verbose {
                    println!("Skipping special file: {}", path.display());
                }
            }
        }

        // All contents match and none are excluded
        Ok(true)
    }

    fn move_entry(&self, src: &Path, dest: &Path, is_dir: bool) -> io::Result<()> {
        if self.dry_run {
            self.handle_dry_run(src, dest, is_dir)
        } else {
            self.handle_move(src, dest, is_dir)
        }
    }

    fn handle_dry_run(&self, src: &Path, dest: &Path, is_dir: bool) -> io::Result<()> {
        if is_dir {
            println!(
                "[DRY RUN] Would move directory {} to {}",
                src.display(),
                dest.display()
            );
        } else {
            println!(
                "[DRY RUN] Would move file {} to {}",
                src.display(),
                dest.display()
            );
        }

        // Update stats using source metadata
        let metadata = match fs::symlink_metadata(src) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("Error accessing metadata for {}: {}", src.display(), e);
                return Ok(());
            }
        };

        self.update_stats(src, &metadata, is_dir)
    }

    fn handle_move(&self, src: &Path, dest: &Path, is_dir: bool) -> io::Result<()> {
        self.create_parent_directories(dest)?;

        if let Err(e) = fs::rename(src, dest) {
            eprintln!(
                "Error moving {} to {}: {}",
                src.display(),
                dest.display(),
                e
            );
            return Err(e); // Propagate the error
        }

        if is_dir {
            if self.verbose {
                println!("Moved directory {} to {}", src.display(), dest.display());
            }
        } else {
            if self.verbose {
                println!("Moved file {} to {}", src.display(), dest.display());
            }
        }

        // Retrieve metadata of the moved file or directory
        let metadata = match fs::symlink_metadata(dest) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("Error accessing metadata for {}: {}", dest.display(), e);
                return Ok(());
            }
        };

        self.update_stats(dest, &metadata, is_dir)
    }

    fn create_parent_directories(&self, dest: &Path) -> io::Result<()> {
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Error creating directory {}: {}", parent.display(), e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn update_stats(&self, path: &Path, metadata: &fs::Metadata, is_dir: bool) -> io::Result<()> {
        if is_dir {
            self.stats.dirs_moved.fetch_add(1, Ordering::SeqCst);
            // Calculate the total size of the directory
            let dir_size = self.calculate_directory_size(path)?;
            self.stats.total_size.fetch_add(dir_size, Ordering::SeqCst);
        } else {
            self.stats.files_moved.fetch_add(1, Ordering::SeqCst);
            self.stats
                .total_size
                .fetch_add(metadata.len(), Ordering::SeqCst);
        }
        Ok(())
    }

    fn calculate_directory_size(&self, path: &Path) -> io::Result<u64> {
        let mut total_size = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                continue; // Skip symbolic links
            }
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += self.calculate_directory_size(&entry.path())?;
            }
        }
        Ok(total_size)
    }
}

#[derive(Default)]
pub struct FileStats {
    pub files_moved: AtomicU64,
    pub dirs_moved: AtomicU64,
    pub total_size: AtomicU64,
}

pub fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = &UNITS[0];

    for next_unit in &UNITS[1..] {
        if size < 1024.0 {
            break;
        }
        size /= 1024.0;
        unit = next_unit;
    }

    format!("{:.2} {}", size, unit)
}
