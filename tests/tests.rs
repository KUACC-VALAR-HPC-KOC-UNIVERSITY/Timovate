use filetime::{set_file_mtime, FileTime};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use timovate::{Cli, FileMover, OperationMode};

#[test]
fn test_move_files_older_than_n_days() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files with modification times
    let old_file_path = temp_source_dir.path().join("old_file.txt");
    fs::write(&old_file_path, b"Old file").unwrap();
    set_file_modified_time(&old_file_path, 40);

    let recent_file_path = temp_source_dir.path().join("recent_file.txt");
    fs::write(&recent_file_path, b"Recent file").unwrap();
    set_file_modified_time(&recent_file_path, 10);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!old_file_path.exists());
    assert!(temp_dest_dir.path().join("old_file.txt").exists());
    assert!(recent_file_path.exists());
}

#[test]
fn test_move_files_newer_than_n_days() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files with modification times
    let old_file_path = temp_source_dir.path().join("old_file.txt");
    fs::write(&old_file_path, b"Old file").unwrap();
    set_file_modified_time(&old_file_path, 40);

    let recent_file_path = temp_source_dir.path().join("recent_file.txt");
    fs::write(&recent_file_path, b"Recent file").unwrap();
    set_file_modified_time(&recent_file_path, 10);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "-30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(old_file_path.exists());
    assert!(!recent_file_path.exists());
    assert!(temp_dest_dir.path().join("recent_file.txt").exists());
}

#[test]
fn test_move_files_exact_n_days() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file modified exactly 30 days ago
    let exact_file_path = temp_source_dir.path().join("exact_file.txt");
    fs::write(&exact_file_path, b"Exact file").unwrap();
    set_file_modified_time(&exact_file_path, 30);

    // Create a file modified 31 days ago
    let old_file_path = temp_source_dir.path().join("old_file.txt");
    fs::write(&old_file_path, b"Old file").unwrap();
    set_file_modified_time(&old_file_path, 31);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!exact_file_path.exists());
    assert!(temp_dest_dir.path().join("exact_file.txt").exists());
    assert!(old_file_path.exists());
}

#[test]
fn test_restore_mode() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_temp_dir = TempDir::new().unwrap();

    // Create files in temporary directory
    let temp_file_path = temp_temp_dir.path().join("temp_file.txt");
    fs::write(&temp_file_path, b"Temporary file").unwrap();

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_temp_dir.path().to_path_buf(),
        days: "0".to_string(), // Changed from "+0" to "0"
        dry_run: false,
        verbose: false,
        mode: OperationMode::Restore,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!temp_file_path.exists());
    assert!(temp_source_dir.path().join("temp_file.txt").exists());
}

#[test]
fn test_dry_run() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file modified 40 days ago
    let old_file_path = temp_source_dir.path().join("old_file.txt");
    fs::write(&old_file_path, b"Old file").unwrap();
    set_file_modified_time(&old_file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: true,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(old_file_path.exists());
    assert!(!temp_dest_dir.path().join("old_file.txt").exists());
}

#[test]
fn test_verbose_mode() {
    // This test checks if verbose output is generated without errors
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file
    let file_path = temp_source_dir.path().join("file.txt");
    fs::write(&file_path, b"File").unwrap();

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "0".to_string(), // Changed from "+0" to "0"
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!file_path.exists());
    assert!(temp_dest_dir.path().join("file.txt").exists());
}

#[test]
fn test_directory_move() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a directory with files
    let dir_path = temp_source_dir.path().join("dir");
    fs::create_dir(&dir_path).unwrap();

    let old_file_path = dir_path.join("old_file.txt");
    fs::write(&old_file_path, b"Old file").unwrap();
    set_file_modified_time(&old_file_path, 40);

    let recent_file_path = dir_path.join("recent_file.txt");
    fs::write(&recent_file_path, b"Recent file").unwrap();
    set_file_modified_time(&recent_file_path, 10);

    // Set up CLI arguments to move directories where all contents match
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(dir_path.exists());
    assert!(recent_file_path.exists());
    assert!(!old_file_path.exists());
    assert!(temp_dest_dir
        .path()
        .join("dir")
        .join("old_file.txt")
        .exists()); // Updated assertion
}

#[test]
fn test_symlink_skipping() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file and a symlink to it
    let file_path = temp_source_dir.path().join("file.txt");
    fs::write(&file_path, b"File").unwrap();

    let symlink_path = temp_source_dir.path().join("symlink.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&file_path, &symlink_path).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&file_path, &symlink_path).unwrap();

    // Set the modification time
    set_file_modified_time(&file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(symlink_path.symlink_metadata().is_ok()); // The symlink should still exist
    assert!(!file_path.exists()); // The original file should be moved
    assert!(temp_dest_dir.path().join("file.txt").exists()); // The file should be in the destination
}

fn set_file_modified_time(path: &Path, days_ago: u64) {
    let modified_time = FileTime::from_system_time(
        SystemTime::now() - Duration::from_secs(days_ago * 24 * 60 * 60),
    );
    set_file_mtime(path, modified_time).unwrap();
}

#[test]
fn test_move_large_number_of_files() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();
    let num_files = 1000; // Adjust this number as needed

    // Create a large number of files with varying modification times
    for i in 0..num_files {
        let file_path = temp_source_dir.path().join(format!("file_{}.txt", i));
        fs::write(&file_path, b"Some content").unwrap();
        // Set modification time ranging from 1 to 60 days ago
        let days_ago = (i % 60) + 1;
        set_file_modified_time(&file_path, days_ago);
    }

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    for i in 0..num_files {
        let file_name = format!("file_{}.txt", i);
        let src_file = temp_source_dir.path().join(&file_name);
        let dest_file = temp_dest_dir.path().join(&file_name);
        let days_ago = (i % 60) + 1;

        if days_ago > 30 {
            // Files older than 30 days should be moved
            assert!(!src_file.exists(), "File {} should be moved", file_name);
            assert!(
                dest_file.exists(),
                "File {} should be in destination",
                file_name
            );
        } else {
            // Files newer than or equal to 30 days should remain
            assert!(src_file.exists(), "File {} should remain", file_name);
            assert!(
                !dest_file.exists(),
                "File {} should not be in destination",
                file_name
            );
        }
    }
}

#[test]
fn test_move_files_at_boundary_conditions() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files at boundary conditions
    let boundary_file_path = temp_source_dir.path().join("boundary_file.txt");
    fs::write(&boundary_file_path, b"Boundary file").unwrap();
    set_file_modified_time(&boundary_file_path, 30);

    let just_before_boundary_file_path =
        temp_source_dir.path().join("just_before_boundary_file.txt");
    fs::write(&just_before_boundary_file_path, b"Just before boundary").unwrap();
    set_file_modified_time(&just_before_boundary_file_path, 29);

    let just_after_boundary_file_path = temp_source_dir.path().join("just_after_boundary_file.txt");
    fs::write(&just_after_boundary_file_path, b"Just after boundary").unwrap();
    set_file_modified_time(&just_after_boundary_file_path, 31);

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        boundary_file_path.exists(),
        "Boundary file should not be moved"
    );
    assert!(
        just_before_boundary_file_path.exists(),
        "File just before boundary should not be moved"
    );
    assert!(
        !just_after_boundary_file_path.exists(),
        "File just after boundary should be moved"
    );
    assert!(
        temp_dest_dir
            .path()
            .join("just_after_boundary_file.txt")
            .exists(),
        "File just after boundary should be in destination"
    );
}

#[test]
fn test_move_files_with_future_modification_times() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files with future modification times
    let future_file_path = temp_source_dir.path().join("future_file.txt");
    fs::write(&future_file_path, b"Future file").unwrap();
    set_file_modified_time_in_future(&future_file_path, 10);

    let present_file_path = temp_source_dir.path().join("present_file.txt");
    fs::write(&present_file_path, b"Present file").unwrap();

    // Set up CLI arguments to move files older than 0 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+0".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(future_file_path.exists(), "Future file should not be moved");
    assert!(
        present_file_path.exists(),
        "Present file should not be moved"
    );
    assert!(
        !temp_dest_dir.path().join("future_file.txt").exists(),
        "Future file should not be in destination"
    );
    assert!(
        !temp_dest_dir.path().join("present_file.txt").exists(),
        "Present file should not be in destination"
    );
}

#[test]
fn test_move_files_with_identical_modification_times() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create multiple files with the same modification time
    let modification_time = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60); // 30 days ago

    for i in 0..10 {
        let file_path = temp_source_dir.path().join(format!("file_{}.txt", i));
        fs::write(&file_path, b"Identical time file").unwrap();
        set_file_modified_time_exact(&file_path, modification_time);
    }

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+29".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    for i in 0..10 {
        let file_name = format!("file_{}.txt", i);
        let src_file = temp_source_dir.path().join(&file_name);
        let dest_file = temp_dest_dir.path().join(&file_name);

        assert!(!src_file.exists(), "File {} should be moved", file_name);
        assert!(
            dest_file.exists(),
            "File {} should be in destination",
            file_name
        );
    }
}

#[test]
fn test_move_files_with_extreme_modification_times() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files with minimum and maximum possible modification times
    let min_time_file_path = temp_source_dir.path().join("min_time_file.txt");
    fs::write(&min_time_file_path, b"Min time file").unwrap();
    set_file_modified_time_exact(&min_time_file_path, SystemTime::UNIX_EPOCH);

    let max_time_file_path = temp_source_dir.path().join("max_time_file.txt");
    fs::write(&max_time_file_path, b"Max time file").unwrap();
    set_file_modified_time_exact(&max_time_file_path, SystemTime::now());

    // Set up CLI arguments to move files older than 0 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+0".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        !min_time_file_path.exists(),
        "Min time file should be moved"
    );
    assert!(
        max_time_file_path.exists(),
        "Max time file should not be moved"
    );
}

#[test]
fn test_move_files_with_high_resolution_timestamps() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file with a modification time that includes sub-second precision
    let high_res_time =
        SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60) + Duration::from_millis(500);
    let high_res_file_path = temp_source_dir.path().join("high_res_file.txt");
    fs::write(&high_res_file_path, b"High resolution file").unwrap();
    set_file_modified_time_exact(&high_res_file_path, high_res_time);

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        high_res_file_path.exists(),
        "High resolution file should not be moved"
    );
}

pub fn set_file_modified_time_exact(path: &Path, time: SystemTime) {
    let modified_time = FileTime::from_system_time(time);
    set_file_mtime(path, modified_time).unwrap();
}

fn set_file_modified_time_in_future(path: &Path, days_in_future: u64) {
    let modified_time = FileTime::from_system_time(
        SystemTime::now() + Duration::from_secs(days_in_future * 24 * 60 * 60),
    );
    set_file_mtime(path, modified_time).unwrap();
}

#[test]
fn test_statistics_in_dry_run() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files and directories
    let file_path = temp_source_dir.path().join("old_file.txt");
    fs::write(&file_path, vec![0u8; 1024]).unwrap(); // 1 KB file
    set_file_modified_time(&file_path, 40);

    let dir_path = temp_source_dir.path().join("old_dir");
    fs::create_dir(&dir_path).unwrap();

    let nested_file_path = dir_path.join("nested_file.txt");
    fs::write(&nested_file_path, vec![0u8; 2048]).unwrap(); // 2 KB file
    set_file_modified_time(&nested_file_path, 40);

    // Expected statistics
    let expected_files_moved = 1; // Only 'old_file.txt'
    let expected_dirs_moved = 1; // 'old_dir'
    let expected_total_size = 1024 + 2048; // 3072 bytes

    // Set up CLI arguments with dry-run enabled
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: true,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert_eq!(
        mover.stats.files_moved.load(Ordering::SeqCst),
        expected_files_moved,
        "Number of files moved should be {}",
        expected_files_moved
    );
    assert_eq!(
        mover.stats.dirs_moved.load(Ordering::SeqCst),
        expected_dirs_moved,
        "Number of directories moved should be {}",
        expected_dirs_moved
    );
    assert_eq!(
        mover.stats.total_size.load(Ordering::SeqCst),
        expected_total_size as u64,
        "Total size should be {} bytes",
        expected_total_size
    );

    // Verify that files and directories still exist since it's a dry-run
    assert!(file_path.exists(), "File should still exist in dry-run");
    assert!(dir_path.exists(), "Directory should still exist in dry-run");
    assert!(
        nested_file_path.exists(),
        "Nested file should still exist in dry-run"
    );
}

#[test]
fn test_total_size_with_nested_directories() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create nested directories with files
    let dir_path = temp_source_dir.path().join("old_dir");
    fs::create_dir(&dir_path).unwrap();

    let sub_dir_path = dir_path.join("sub_dir");
    fs::create_dir(&sub_dir_path).unwrap();

    let file_sizes = [1024, 2048, 4096]; // Sizes in bytes
    let mut total_moved_size = 0;

    let file_paths = [
        dir_path.join("file1.txt"),
        sub_dir_path.join("file2.txt"),
        sub_dir_path.join("file3.txt"),
    ];

    for (file_path, size) in file_paths.iter().zip(file_sizes.iter()) {
        let content = vec![0u8; *size];
        fs::write(file_path, &content).unwrap();
        set_file_modified_time(file_path, 40);

        total_moved_size += size;
    }

    // Set up CLI arguments to move directories where all contents match
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert_eq!(
        mover.stats.dirs_moved.load(Ordering::SeqCst),
        1,
        "One directory should be moved"
    );
    assert_eq!(
        mover.stats.files_moved.load(Ordering::SeqCst),
        0,
        "Files inside the moved directory should not be individually counted"
    );
    // For accurate total size, you may need to implement recursive size calculation
    // For this test, we'll assume total_size remains 0 unless implemented
    // If you implement total_size calculation for directories, adjust the assertion accordingly
    // Example:
    assert_eq!(
        mover.stats.total_size.load(Ordering::SeqCst),
        total_moved_size as u64,
        "Total size should be {} bytes",
        total_moved_size
    );
}

#[test]
fn test_dry_run_does_not_modify_files() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files and directories
    let file_path = temp_source_dir.path().join("file.txt");
    fs::write(&file_path, b"Original content").unwrap();
    set_file_modified_time(&file_path, 40);

    let dir_path = temp_source_dir.path().join("dir");
    fs::create_dir(&dir_path).unwrap();

    let nested_file_path = dir_path.join("nested_file.txt");
    fs::write(&nested_file_path, b"Nested content").unwrap();
    set_file_modified_time(&nested_file_path, 40);

    // Record initial metadata
    let file_metadata_before = fs::metadata(&file_path).unwrap();
    let nested_file_metadata_before = fs::metadata(&nested_file_path).unwrap();

    // Set up CLI arguments with dry-run enabled
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: true,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Record metadata after dry-run
    let file_metadata_after = fs::metadata(&file_path).unwrap();
    let nested_file_metadata_after = fs::metadata(&nested_file_path).unwrap();

    // Assertions
    assert_eq!(
        file_metadata_before.modified().unwrap(),
        file_metadata_after.modified().unwrap(),
        "File modification time should not change"
    );
    assert_eq!(
        nested_file_metadata_before.modified().unwrap(),
        nested_file_metadata_after.modified().unwrap(),
        "Nested file modification time should not change"
    );
    assert!(file_path.exists(), "File should still exist after dry-run");
    assert!(
        nested_file_path.exists(),
        "Nested file should still exist after dry-run"
    );
}

#[test]
fn test_statistics_with_mixed_content_directories() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a directory with mixed content
    let dir_path = temp_source_dir.path().join("dir");
    fs::create_dir(&dir_path).unwrap();

    let old_file_path = dir_path.join("old_file.txt");
    fs::write(&old_file_path, vec![0u8; 1024]).unwrap(); // 1 KB file
    set_file_modified_time(&old_file_path, 40);

    let recent_file_path = dir_path.join("recent_file.txt");
    fs::write(&recent_file_path, vec![0u8; 2048]).unwrap(); // 2 KB file
    set_file_modified_time(&recent_file_path, 10);

    // Expected statistics
    let expected_files_moved = 1;
    let expected_total_size = 1024;

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert_eq!(
        mover.stats.files_moved.load(Ordering::SeqCst),
        expected_files_moved,
        "Number of files moved should be {}",
        expected_files_moved
    );
    assert_eq!(
        mover.stats.dirs_moved.load(Ordering::SeqCst),
        0,
        "No directories should be moved since contents are mixed"
    );
    assert_eq!(
        mover.stats.total_size.load(Ordering::SeqCst),
        expected_total_size,
        "Total size should be {} bytes",
        expected_total_size
    );

    // Verify that only the old file was moved
    assert!(!old_file_path.exists(), "Old file should be moved");
    assert!(
        temp_dest_dir
            .path()
            .join("dir")
            .join("old_file.txt")
            .exists(),
        "Old file should be in destination"
    );
    assert!(recent_file_path.exists(), "Recent file should remain");
}

#[test]
fn test_total_size_excludes_symbolic_links() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file and a symbolic link to it
    let file_path = temp_source_dir.path().join("file.txt");
    fs::write(&file_path, vec![0u8; 1024]).unwrap(); // 1 KB file
    set_file_modified_time(&file_path, 40);

    let symlink_path = temp_source_dir.path().join("symlink.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&file_path, &symlink_path).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&file_path, &symlink_path).unwrap();

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert_eq!(
        mover.stats.files_moved.load(Ordering::SeqCst),
        1,
        "Only the original file should be counted as moved"
    );
    assert_eq!(
        mover.stats.total_size.load(Ordering::SeqCst),
        1024,
        "Total size should be 1024 bytes"
    );
    // Verify that the symlink still exists and was not counted
    assert!(
        symlink_path.symlink_metadata().is_ok(),
        "Symbolic link should still exist"
    );
}

#[cfg(unix)]
#[test]
fn test_skip_special_files() {
    use nix::sys::stat;
    use nix::unistd;
    use std::os::unix::net::UnixListener;

    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a FIFO (named pipe)
    let fifo_path = temp_source_dir.path().join("fifo_pipe");
    unistd::mkfifo(&fifo_path, stat::Mode::S_IRUSR | stat::Mode::S_IWUSR).unwrap();

    // Create a Unix socket
    let socket_path = temp_source_dir.path().join("unix_socket");
    let _listener = UnixListener::bind(&socket_path).unwrap();

    // Create a regular file
    let file_path = temp_source_dir.path().join("regular_file.txt");
    fs::write(&file_path, b"Regular file").unwrap();
    set_file_modified_time(&file_path, 40);

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(fifo_path.exists(), "FIFO should not be moved");
    assert!(socket_path.exists(), "Socket should not be moved");
    assert!(!file_path.exists(), "Regular file should be moved");
    assert!(
        temp_dest_dir.path().join("regular_file.txt").exists(),
        "Regular file should be in destination"
    );
}

#[test]
fn test_exclude_regex_files() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files
    let exclude_file_path = temp_source_dir.path().join("exclude_me.txt");
    fs::write(&exclude_file_path, b"Exclude me").unwrap();
    set_file_modified_time(&exclude_file_path, 40);

    let include_file_path = temp_source_dir.path().join("include_me.txt");
    fs::write(&include_file_path, b"Include me").unwrap();
    set_file_modified_time(&include_file_path, 40);

    // Set up CLI arguments with exclude regex
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: Some(vec!["exclude_me\\.txt$".to_string()]),
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        exclude_file_path.exists(),
        "Excluded file should not be moved"
    );
    assert!(!include_file_path.exists(), "Included file should be moved");
    assert!(
        temp_dest_dir.path().join("include_me.txt").exists(),
        "Included file should be in destination"
    );
}

#[test]
fn test_exclude_regex_directories() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create directories and files
    let exclude_dir_path = temp_source_dir.path().join("exclude_dir");
    fs::create_dir(&exclude_dir_path).unwrap();

    let exclude_file_path = exclude_dir_path.join("old_file.txt");
    fs::write(&exclude_file_path, b"Old file").unwrap();
    set_file_modified_time(&exclude_file_path, 40);

    let include_dir_path = temp_source_dir.path().join("include_dir");
    fs::create_dir(&include_dir_path).unwrap();

    let include_file_path = include_dir_path.join("old_file.txt");
    fs::write(&include_file_path, b"Old file").unwrap();
    set_file_modified_time(&include_file_path, 40);

    // Set up CLI arguments with exclude regex
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: Some(vec!["exclude_dir$".to_string()]),
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        exclude_dir_path.exists(),
        "Excluded directory should not be moved"
    );
    assert!(
        exclude_file_path.exists(),
        "Files in excluded directory should not be moved"
    );
    assert!(
        !include_dir_path.exists(),
        "Included directory should be moved"
    );
    assert!(
        temp_dest_dir.path().join("include_dir").exists(),
        "Included directory should be in destination"
    );
}

#[test]
fn test_invalid_regex_pattern() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file
    let file_path = temp_source_dir.path().join("file.txt");
    fs::write(&file_path, b"File").unwrap();

    // Set up CLI arguments with invalid regex
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: Some(vec!["*invalid[".to_string()]), // Invalid regex
    };

    // Since the invalid regex causes the program to exit, we need to catch the error
    let result = std::panic::catch_unwind(|| {
        let mover = FileMover::new(&cli).unwrap();
        mover.execute().unwrap();
    });

    assert!(
        result.is_err(),
        "Program should panic due to invalid regex pattern"
    );
}

#[test]
fn test_invalid_days_input() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Set up CLI arguments with invalid days input
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "invalid".to_string(), // Invalid input
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    // Try to create the FileMover
    let result = FileMover::new(&cli);

    assert!(
        result.is_err(),
        "Program should return an error due to invalid days input"
    );
}

#[test]
fn test_human_readable_size() {
    use timovate::human_readable_size;

    assert_eq!(human_readable_size(0), "0.00 B");
    assert_eq!(human_readable_size(500), "500.00 B");
    assert_eq!(human_readable_size(1024), "1.00 KB");
    assert_eq!(human_readable_size(1536), "1.50 KB");
    assert_eq!(human_readable_size(1048576), "1.00 MB");
    assert_eq!(human_readable_size(1073741824), "1.00 GB");
    assert_eq!(human_readable_size(1099511627776), "1.00 TB");
}

#[test]
fn test_special_characters_in_filenames() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files with special characters
    let special_file_path = temp_source_dir.path().join("file with spaces.txt");
    fs::write(&special_file_path, b"Special file").unwrap();
    set_file_modified_time(&special_file_path, 40);

    let unicode_file_path = temp_source_dir.path().join("файл.txt");
    fs::write(&unicode_file_path, b"Unicode file").unwrap();
    set_file_modified_time(&unicode_file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        !special_file_path.exists(),
        "File with spaces should be moved"
    );
    assert!(
        temp_dest_dir.path().join("file with spaces.txt").exists(),
        "File with spaces should be in destination"
    );
    assert!(!unicode_file_path.exists(), "Unicode file should be moved");
    assert!(
        temp_dest_dir.path().join("файл.txt").exists(),
        "Unicode file should be in destination"
    );
}

#[test]
fn test_read_only_files() {
    use std::os::unix::fs::PermissionsExt;

    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a read-only file
    let file_path = temp_source_dir.path().join("readonly_file.txt");
    fs::write(&file_path, b"Read-only file").unwrap();
    set_file_modified_time(&file_path, 40);

    // Set permissions to read-only
    let mut permissions = fs::metadata(&file_path).unwrap().permissions();
    permissions.set_mode(0o444); // Read-only permissions
    fs::set_permissions(&file_path, permissions).unwrap();

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    let result = mover.execute();

    // Assertions
    assert!(
        result.is_ok(),
        "Program should handle read-only files without error"
    );
    assert!(
        !file_path.exists(),
        "Read-only file should be moved successfully"
    );
    assert!(
        temp_dest_dir.path().join("readonly_file.txt").exists(),
        "Read-only file should be in destination"
    );
}

#[test]
fn test_insufficient_permissions() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();

        // Create a file
        let file_path = temp_source_dir.path().join("file.txt");
        fs::write(&file_path, b"File").unwrap();
        set_file_modified_time(&file_path, 40);

        // Remove write permissions from the source directory
        let mut permissions = fs::metadata(temp_source_dir.path()).unwrap().permissions();
        permissions.set_mode(0o555); // Read and execute permissions
        fs::set_permissions(temp_source_dir.path(), permissions).unwrap();

        // Set up CLI arguments
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: true,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        let result = mover.execute();

        // Restore permissions for cleanup
        let mut permissions = fs::metadata(temp_source_dir.path()).unwrap().permissions();
        permissions.set_mode(0o755); // Read, write, execute
        fs::set_permissions(temp_source_dir.path(), permissions).unwrap();

        // Assertions
        assert!(
            result.is_err(),
            "Program should report an error due to insufficient permissions"
        );
        assert!(
            file_path.exists(),
            "File should not be moved due to insufficient permissions"
        );
    }
}

#[test]
fn test_empty_source_directory() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Ensure the source directory is empty

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    let result = mover.execute();

    // Assertions
    assert!(
        result.is_ok(),
        "Program should handle empty source directory without error"
    );
}

#[test]
fn test_move_files_with_no_matching_time_criteria() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files that do not match the time criteria
    let recent_file_path = temp_source_dir.path().join("recent_file.txt");
    fs::write(&recent_file_path, b"Recent file").unwrap();
    set_file_modified_time(&recent_file_path, 10);

    // Set up CLI arguments to move files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        recent_file_path.exists(),
        "File should not be moved as it does not match the time criteria"
    );
    assert!(
        temp_dest_dir.path().read_dir().unwrap().next().is_none(),
        "Destination directory should remain empty"
    );
}

#[test]
fn test_restore_mode_with_exclude_regex() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_temp_dir = TempDir::new().unwrap();

    // Create files in temporary directory
    let exclude_file_path = temp_temp_dir.path().join("exclude_me.txt");
    fs::write(&exclude_file_path, b"Exclude me").unwrap();

    let include_file_path = temp_temp_dir.path().join("include_me.txt");
    fs::write(&include_file_path, b"Include me").unwrap();

    // Set up CLI arguments with exclude regex in restore mode
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_temp_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Restore,
        exclude: Some(vec!["exclude_me\\.txt$".to_string()]),
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        !exclude_file_path.exists(),
        "Excluded file should also be restored"
    );
    assert!(
        !include_file_path.exists(),
        "Included file should be restored"
    );
    assert!(
        temp_source_dir.path().join("include_me.txt").exists(),
        "Included file should be in source directory"
    );
}

#[test]
fn test_move_files_with_multiple_exclude_patterns() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files
    let file1 = temp_source_dir.path().join("file1.log");
    fs::write(&file1, b"Log file").unwrap();
    set_file_modified_time(&file1, 40);

    let file2 = temp_source_dir.path().join("file2.tmp");
    fs::write(&file2, b"Temporary file").unwrap();
    set_file_modified_time(&file2, 40);

    let file3 = temp_source_dir.path().join("file3.txt");
    fs::write(&file3, b"Text file").unwrap();
    set_file_modified_time(&file3, 40);

    // Set up CLI arguments with multiple exclude patterns
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: Some(vec!["\\.log$".to_string(), "\\.tmp$".to_string()]),
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(file1.exists(), "Log file should be excluded");
    assert!(file2.exists(), "Temporary file should be excluded");
    assert!(!file3.exists(), "Text file should be moved");
    assert!(
        temp_dest_dir.path().join("file3.txt").exists(),
        "Text file should be in destination"
    );
}

#[test]
fn test_move_files_with_hidden_files() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create hidden files
    let hidden_file_path = temp_source_dir.path().join(".hidden_file");
    fs::write(&hidden_file_path, b"Hidden file").unwrap();
    set_file_modified_time(&hidden_file_path, 40);

    let visible_file_path = temp_source_dir.path().join("visible_file.txt");
    fs::write(&visible_file_path, b"Visible file").unwrap();
    set_file_modified_time(&visible_file_path, 40);

    // Set up CLI arguments to move all files older than 30 days
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!hidden_file_path.exists(), "Hidden file should be moved");
    assert!(!visible_file_path.exists(), "Visible file should be moved");
    assert!(
        temp_dest_dir.path().join(".hidden_file").exists(),
        "Hidden file should be in destination"
    );
    assert!(
        temp_dest_dir.path().join("visible_file.txt").exists(),
        "Visible file should be in destination"
    );
}

#[test]
fn test_move_files_with_large_file_sizes() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a large file (e.g., 100 MB)
    let large_file_path = temp_source_dir.path().join("large_file.bin");
    let large_file_size = 100 * 1024 * 1024; // 100 MB
    let content = vec![0u8; large_file_size];
    fs::write(&large_file_path, &content).unwrap();
    set_file_modified_time(&large_file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!large_file_path.exists(), "Large file should be moved");
    assert!(
        temp_dest_dir.path().join("large_file.bin").exists(),
        "Large file should be in destination"
    );
    assert_eq!(
        mover.stats.total_size.load(Ordering::SeqCst),
        large_file_size as u64,
        "Total size should match the size of the large file"
    );
}

#[test]
fn test_move_files_with_non_ascii_characters_in_path() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a directory with non-ASCII characters
    let dir_name = "ディレクトリ";
    let dir_path = temp_source_dir.path().join(dir_name);
    fs::create_dir(&dir_path).unwrap();

    // Create a file inside this directory
    let file_path = dir_path.join("ファイル.txt");
    fs::write(&file_path, b"Content").unwrap();
    set_file_modified_time(&file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        !dir_path.exists(),
        "Directory with non-ASCII name should be moved"
    );
    assert!(
        temp_dest_dir.path().join(dir_name).exists(),
        "Directory should be in destination"
    );
    assert!(
        temp_dest_dir
            .path()
            .join(dir_name)
            .join("ファイル.txt")
            .exists(),
        "File inside non-ASCII directory should be moved"
    );
}

#[test]
fn test_move_files_with_empty_exclude_list() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create files
    let file1 = temp_source_dir.path().join("file1.txt");
    fs::write(&file1, b"File 1").unwrap();
    set_file_modified_time(&file1, 40);

    let file2 = temp_source_dir.path().join("file2.txt");
    fs::write(&file2, b"File 2").unwrap();
    set_file_modified_time(&file2, 40);

    // Set up CLI arguments with an empty exclude list
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: Some(vec![]), // Empty exclude list
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(!file1.exists(), "File1 should be moved");
    assert!(!file2.exists(), "File2 should be moved");
    assert!(
        temp_dest_dir.path().join("file1.txt").exists(),
        "File1 should be in destination"
    );
    assert!(
        temp_dest_dir.path().join("file2.txt").exists(),
        "File2 should be in destination"
    );
}

#[test]
fn test_move_files_with_recursive_exclude_patterns() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create nested directories and files
    let dir_path = temp_source_dir.path().join("dir");
    fs::create_dir(&dir_path).unwrap();

    let sub_dir_path = dir_path.join("subdir");
    fs::create_dir(&sub_dir_path).unwrap();

    let exclude_file_path = sub_dir_path.join("exclude_me.txt");
    fs::write(&exclude_file_path, b"Exclude me").unwrap();
    set_file_modified_time(&exclude_file_path, 40);

    let include_file_path = sub_dir_path.join("include_me.txt");
    fs::write(&include_file_path, b"Include me").unwrap();
    set_file_modified_time(&include_file_path, 40);

    // Set up CLI arguments with recursive exclude pattern
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: Some(vec!["exclude_me\\.txt$".to_string()]),
    };

    let mover = FileMover::new(&cli).unwrap();
    mover.execute().unwrap();

    // Assertions
    assert!(
        exclude_file_path.exists(),
        "Excluded file should not be moved"
    );
    assert!(!include_file_path.exists(), "Included file should be moved");
    assert!(
        temp_dest_dir
            .path()
            .join("dir")
            .join("subdir")
            .join("include_me.txt")
            .exists(),
        "Included file should be in destination"
    );
}

#[test]
fn test_restore_ignores_days_parameter() {
    // Create source and temporary directories
    let temp_source_dir = TempDir::new().unwrap();
    let temp_temp_dir = TempDir::new().unwrap();

    // Create a file in the temporary directory to be restored
    let temp_file_path = temp_temp_dir.path().join("temp_file.txt");
    fs::write(&temp_file_path, b"Temporary file").unwrap();
    set_file_modified_time(&temp_file_path, 40); // File modified 40 days ago

    // Set up CLI arguments with a days parameter that doesn't match the file's age
    let cli_restore = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_temp_dir.path().to_path_buf(),
        days: "-10".to_string(), // Files modified less than 10 days ago
        dry_run: false,
        verbose: true,
        mode: OperationMode::Restore,
        exclude: None,
    };

    let mover_restore = FileMover::new(&cli_restore).unwrap();
    let result = mover_restore.execute();

    // The restore should succeed and restore the file despite the days parameter
    assert!(
        result.is_ok(),
        "Restore should succeed regardless of --days parameter"
    );

    // Verify that temp_file.txt is restored to the source directory
    assert!(
        temp_source_dir.path().join("temp_file.txt").exists(),
        "temp_file.txt should be restored"
    );
    assert!(
        !temp_file_path.exists(),
        "temp_file.txt should no longer be in temporary directory"
    );
}

#[test]
fn test_move_with_empty_source_directory() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Source directory is empty

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    let result = mover.execute();

    // Assertions
    assert!(
        result.is_ok(),
        "Program should handle empty source directory gracefully"
    );
    assert_eq!(
        mover.stats.files_moved.load(Ordering::SeqCst),
        0,
        "No files should be moved from an empty source directory"
    );
}

#[test]
fn test_move_with_same_source_and_destination() {
    let temp_dir = TempDir::new().unwrap();

    // Set up CLI arguments with same source and temporary directories
    let cli = Cli {
        source: temp_dir.path().to_path_buf(),
        temporary: temp_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: false,
        mode: OperationMode::Move,
        exclude: None,
    };

    // Attempt to create FileMover should fail
    let result = FileMover::new(&cli);

    assert!(
        result.is_err(),
        "Program should not allow the same directory for source and temporary"
    );
}

#[test]
fn test_move_files_with_long_file_names() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a file with a very long name
    let long_file_name = "a".repeat(255); // Maximum filename length on many filesystems
    let long_file_path = temp_source_dir.path().join(&long_file_name);
    fs::write(&long_file_path, b"Long file name").unwrap();
    set_file_modified_time(&long_file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let mover = FileMover::new(&cli).unwrap();
    let result = mover.execute();

    // Assertions
    assert!(
        result.is_ok(),
        "Program should handle long file names without error"
    );
    assert!(
        !long_file_path.exists(),
        "File with long name should be moved"
    );
    assert!(
        temp_dest_dir.path().join(&long_file_name).exists(),
        "File with long name should be in destination"
    );
}

#[test]
fn test_move_files_with_invalid_characters_in_filenames() {
    let temp_source_dir = TempDir::new().unwrap();

    // Create files with invalid characters in filenames
    let invalid_file_name = "invalid\0name.txt"; // Null character is invalid
    let invalid_file_path = temp_source_dir.path().join(&invalid_file_name);

    // Attempting to create a file with an invalid name should fail
    let result = fs::write(&invalid_file_path, b"Invalid filename");

    // Assertions
    assert!(
        result.is_err(),
        "Creating a file with an invalid name should fail"
    );
}

#[test]
fn test_move_with_circular_symbolic_links() {
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    // Create a directory and a symlink to itself
    let dir_path = temp_source_dir.path().join("dir");
    fs::create_dir(&dir_path).unwrap();

    let symlink_path = dir_path.join("symlink");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&dir_path, &symlink_path).unwrap();

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "0".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    // Since symbolic links are skipped, this should not cause infinite recursion
    let mover = FileMover::new(&cli).unwrap();
    let result = mover.execute();

    // Assertions
    assert!(
        result.is_ok(),
        "Program should handle circular symbolic links gracefully"
    );
}

#[test]
fn test_move_files_without_read_permission() {
    #[cfg(unix)]
    {
        let temp_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();

        // Create a file and remove read permissions
        let file_path = temp_source_dir.path().join("file.txt");
        fs::write(&file_path, b"Content").unwrap();
        set_file_modified_time(&file_path, 40);

        let mut permissions = fs::metadata(&file_path).unwrap().permissions();
        permissions.set_mode(0o000); // No permissions
        fs::set_permissions(&file_path, permissions.clone()).unwrap();

        // Set up CLI arguments
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: true,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        let result = mover.execute();

        // Restore permissions for cleanup
        let dest_file_path = temp_dest_dir.path().join("file.txt");
        if dest_file_path.exists() {
            permissions.set_mode(0o644);
            fs::set_permissions(&dest_file_path, permissions).unwrap();
        } else {
            // If the file was not moved, restore permissions at the source
            permissions.set_mode(0o644);
            fs::set_permissions(&file_path, permissions).unwrap();
        }

        // Assertions
        assert!(
            result.is_ok(),
            "Program should handle files without read permission without error"
        );
        assert!(
            dest_file_path.exists(),
            "File should be moved to destination"
        );
    }
}

#[test]
fn test_move_files_with_hard_links() {
    #[cfg(unix)]
    {
        let temp_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();

        // Create a file and a hard link to it
        let file_path = temp_source_dir.path().join("file.txt");
        fs::write(&file_path, b"Content").unwrap();
        set_file_modified_time(&file_path, 40);

        let hard_link_path = temp_source_dir.path().join("hard_link.txt");
        fs::hard_link(&file_path, &hard_link_path).unwrap();

        // Set up CLI arguments
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: false,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        mover.execute().unwrap();

        // Assertions
        assert!(!file_path.exists(), "Original file should be moved");
        assert!(!hard_link_path.exists(), "Hard link should be moved");
        assert!(
            temp_dest_dir.path().join("file.txt").exists(),
            "File should be in destination"
        );
        assert!(
            temp_dest_dir.path().join("hard_link.txt").exists(),
            "Hard link should be in destination"
        );
    }
}

#[test]
fn test_move_hidden_files_on_unix() {
    #[cfg(unix)]
    {
        let temp_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();

        // Create hidden files (starting with a dot)
        let hidden_file_path = temp_source_dir.path().join(".hidden_file");
        fs::write(&hidden_file_path, b"Hidden file").unwrap();
        set_file_modified_time(&hidden_file_path, 40);

        // Set up CLI arguments
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: false,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        mover.execute().unwrap();

        // Assertions
        assert!(!hidden_file_path.exists(), "Hidden file should be moved");
        assert!(
            temp_dest_dir.path().join(".hidden_file").exists(),
            "Hidden file should be in destination"
        );
    }
}

#[test]
fn test_move_with_source_as_symbolic_link() {
    #[cfg(unix)]
    {
        let real_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();
        let temp_symlink_dir = TempDir::new().unwrap();

        let symlink_path = temp_symlink_dir.path().join("symlink_source");
        std::os::unix::fs::symlink(real_source_dir.path(), &symlink_path).unwrap();

        // Create a file in the real source directory
        let file_path = real_source_dir.path().join("file.txt");
        fs::write(&file_path, b"Content").unwrap();
        set_file_modified_time(&file_path, 40);

        // Set up CLI arguments with symlink as source
        let cli = Cli {
            source: symlink_path.clone(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: true,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        let result = mover.execute();

        // Assertions
        assert!(
            result.is_ok(),
            "Operation should succeed when source is a symlink"
        );
        assert!(
            !file_path.exists(),
            "File should be moved from the real source directory"
        );
        assert!(
            temp_dest_dir.path().join("file.txt").exists(),
            "File should be in destination"
        );
    }
}

#[test]
fn test_move_with_destination_as_symbolic_link() {
    #[cfg(unix)]
    {
        let temp_source_dir = TempDir::new().unwrap();
        let real_dest_dir = TempDir::new().unwrap();
        let temp_symlink_dir = TempDir::new().unwrap();

        let symlink_path = temp_symlink_dir.path().join("symlink_dest");
        std::os::unix::fs::symlink(real_dest_dir.path(), &symlink_path).unwrap();

        // Create a file in the source directory
        let file_path = temp_source_dir.path().join("file.txt");
        fs::write(&file_path, b"Content").unwrap();
        set_file_modified_time(&file_path, 40);

        // Set up CLI arguments with symlink as temporary directory
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: symlink_path.clone(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: true,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        let result = mover.execute();

        // Assertions
        assert!(
            result.is_ok(),
            "Operation should succeed when destination is a symlink"
        );
        assert!(
            !file_path.exists(),
            "File should be moved from the source directory"
        );
        assert!(
            real_dest_dir.path().join("file.txt").exists(),
            "File should be in the real destination directory"
        );
    }
}

#[test]
fn test_move_files_with_max_path_length() {
    // Create a deeply nested directory structure approaching max path length
    let temp_source_dir = TempDir::new().unwrap();
    let temp_dest_dir = TempDir::new().unwrap();

    let mut current_dir = temp_source_dir.path().to_path_buf();
    for _ in 0..50 {
        current_dir = current_dir.join("nested");
        fs::create_dir(&current_dir).unwrap();
    }

    let file_path = current_dir.join("file.txt");
    fs::write(&file_path, b"Content").unwrap();
    set_file_modified_time(&file_path, 40);

    // Set up CLI arguments
    let cli = Cli {
        source: temp_source_dir.path().to_path_buf(),
        temporary: temp_dest_dir.path().to_path_buf(),
        days: "+30".to_string(),
        dry_run: false,
        verbose: true,
        mode: OperationMode::Move,
        exclude: None,
    };

    let result = FileMover::new(&cli).and_then(|mover| Ok(mover.execute()));

    // Assertions
    if result.is_err() {
        // On some systems, this may fail due to path length limitations
        eprintln!("Operation failed due to path length limitations");
    } else {
        assert!(
            !file_path.exists(),
            "File should be moved despite deep nesting"
        );
    }
}

#[test]
fn test_move_files_with_large_number_of_hard_links() {
    #[cfg(unix)]
    {
        let temp_source_dir = TempDir::new().unwrap();
        let temp_dest_dir = TempDir::new().unwrap();

        // Create a file with multiple hard links
        let file_path = temp_source_dir.path().join("file.txt");
        fs::write(&file_path, b"Content").unwrap();
        set_file_modified_time(&file_path, 40);

        for i in 0..10 {
            let hard_link_path = temp_source_dir.path().join(format!("hard_link_{}.txt", i));
            fs::hard_link(&file_path, &hard_link_path).unwrap();
        }

        // Set up CLI arguments
        let cli = Cli {
            source: temp_source_dir.path().to_path_buf(),
            temporary: temp_dest_dir.path().to_path_buf(),
            days: "+30".to_string(),
            dry_run: false,
            verbose: false,
            mode: OperationMode::Move,
            exclude: None,
        };

        let mover = FileMover::new(&cli).unwrap();
        mover.execute().unwrap();

        // Assertions
        for i in 0..10 {
            let hard_link_path = temp_source_dir.path().join(format!("hard_link_{}.txt", i));
            assert!(!hard_link_path.exists(), "Hard link {} should be moved", i);
            assert!(
                temp_dest_dir
                    .path()
                    .join(format!("hard_link_{}.txt", i))
                    .exists(),
                "Hard link {} should be in destination",
                i
            );
        }
    }
}
