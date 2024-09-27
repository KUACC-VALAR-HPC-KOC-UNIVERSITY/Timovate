# Timovate

Timovate is a command-line utility written in Rust that moves or restores files based on their modification time. It's designed to help you manage files that are older or newer than a specified number of days, similar to the `-mtime` option in the Unix `find` command.

## Features

- **Move Files Based on Age**: Move files that are older or newer than a specified number of days.
- **Restore Files**: Easily restore moved files back to their original location.
- **Dry Run Mode**: Preview the files that would be moved without making any changes.
- **Verbose Output**: Get detailed information about the operations being performed.
- **Exclude Patterns**: Use regular expressions to exclude specific files or directories.
- **Parallel Processing**: Efficiently process files using parallelism for better performance.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Command-Line Options](#command-line-options)
  - [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Prerequisites

- **Rust**: Timovate is written in Rust. Ensure you have Rust and Cargo installed. You can install Rust using [rustup](https://rustup.rs/).

### Build from Source

1. **Clone the Repository**

   ```bash
   git clone https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/Timovate/new/main?filename=README.md
   cd timovate
   ```

2. **Build the Executable**

   ```bash
   cargo build --release
   ```

3. **Install**

   Optionally, you can install Timovate globally:

   ```bash
   cargo install --path .
   ```

## Usage

```bash
timovate [OPTIONS]
```

### Command-Line Options

| Option                  | Description                                                                                              | Default    |
| ----------------------- | -------------------------------------------------------------------------------------------------------- | ---------- |
| `-s`, `--source`        | Source directory to search for files or restore to.                                                      | *Required* |
| `-t`, `--temporary`     | Directory to move files to or restore from.                                                              | *Required* |
| `--days`                | Time criteria for moving files (e.g., `+30`, `-15`, `0` days), similar to `find`'s `-mtime`.             | `+30`      |
| `--dry-run`             | Perform a dry run without moving files.                                                                  | `false`    |
| `-v`, `--verbose`       | Enable verbose mode to get detailed output.                                                              | `false`    |
| `-m`, `--mode`          | Operation mode: `move` or `restore`.                                                                     | `move`     |
| `-e`, `--exclude`       | Regex pattern(s) to exclude files or directories. Can be specified multiple times for multiple patterns. | None       |

### Time Criteria Syntax

- `+N`: Matches files modified more than `N` days ago.
- `-N`: Matches files modified less than `N` days ago.
- `N`: Matches files modified exactly `N` days ago.

### Examples

#### Move Files Older Than 30 Days

```bash
timovate --source /path/to/source --temporary /path/to/temporary --days +30
```

#### Restore Files from Temporary Directory

```bash
timovate --source /path/to/source --temporary /path/to/temporary --mode restore
```

#### Dry Run to See What Would Be Moved

```bash
timovate --source /path/to/source --temporary /path/to/temporary --days +30 --dry-run
```

#### Move Files Newer Than 10 Days with Verbose Output

```bash
timovate --source /path/to/source --temporary /path/to/temporary --days -10 --verbose
```

#### Exclude Certain Files or Directories

```bash
timovate --source /path/to/source --temporary /path/to/temporary --days +30 --exclude ".*\.log" --exclude "backup_.*"
```

This command excludes files ending with `.log` and directories starting with `backup_`.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
