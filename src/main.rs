// Copyright (C) 2020 Francois Marier
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]

mod main_test;

use glob::glob;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process;

const GLOBAL_CONFIG: &str = "/etc/safe-rm.conf";
const LOCAL_GLOBAL_CONFIG: &str = "/usr/local/etc/safe-rm.conf";
const USER_CONFIG: &str = ".config/safe-rm";
const LEGACY_USER_CONFIG: &str = ".safe-rm";

const REAL_RM: &str = "/bin/rm";

const DEFAULT_PATHS: &[&str] = &[
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/home",
    "/initrd",
    "/lib",
    "/lib32",
    "/lib64",
    "/proc",
    "/root",
    "/sbin",
    "/sys",
    "/usr",
    "/usr/bin",
    "/usr/include",
    "/usr/lib",
    "/usr/local",
    "/usr/local/bin",
    "/usr/local/include",
    "/usr/local/sbin",
    "/usr/local/share",
    "/usr/sbin",
    "/usr/share",
    "/usr/src",
    "/var",
];

const MAX_GLOB_EXPANSION: usize = 256;

fn read_config<P: AsRef<Path>>(filename: P) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !filename.as_ref().exists() {
        // Not all config files are expected to be present.
        // If they're missing, we silently skip them.
        return Some(paths);
    }
    let f = File::open(&filename).ok().or_else(|| {
        println!(
            "safe-rm: Could not open configuration file: {}",
            filename.as_ref().display()
        );
        None
    })?;

    let reader = io::BufReader::new(f);
    paths.extend(
        reader
            .lines()
            .filter_map(|line| parse_line(filename.as_ref().display(), line))
            .flatten(),
    );
    Some(paths)
}

fn parse_line<D: std::fmt::Display>(
    filename: D,
    line_result: io::Result<String>,
) -> Option<Vec<PathBuf>> {
    let line = line_result.ok().or_else(|| {
        println!("safe-rm: Ignoring unreadable line in {}.", filename);
        None
    })?;
    let entries = glob(&line).ok().or_else(|| {
        println!(
            "safe-rm: Invalid glob pattern \"{}\" found in {} and ignored.",
            line, filename
        );
        None
    })?;

    let mut paths = Vec::new();

    for entry in entries {
        match entry {
            Ok(path) => {
                if paths.len() >= MAX_GLOB_EXPANSION {
                    println!(
                        "safe-rm: Glob \"{}\" found in {} expands to more than {} paths. Ignoring the rest.",
                        line, filename, MAX_GLOB_EXPANSION
                    );
                    return Some(paths);
                }
                paths.push(path);
            }
            Err(_) => println!(
                "safe-rm: Ignored unreadable path while expanding glob \"{}\" from {}.",
                line, filename
            ),
        }
    }

    Some(paths)
}

fn symlink_canonicalize(path: &Path) -> Option<PathBuf> {
    // Relative paths need to be prefixed by "./" to have a parent dir.
    let mut explicit_path = path.to_path_buf();
    if explicit_path.is_relative() {
        explicit_path = Path::new(".").join(path);
    }

    // Convert from relative to absolute path but don't follow the symlink.
    // We do this by:
    // 1. splitting directory and base file name
    // 2. canonicalizing the directory
    // 3. recombining directory and file name
    let parent: Option<PathBuf> = match explicit_path.parent() {
        Some(dir) => match dir.canonicalize() {
            Ok(normalized_parent) => Some(normalized_parent),
            Err(_) => None,
        },
        None => Some(PathBuf::from("/")),
    };
    return match parent {
        Some(dir) => match path.file_name() {
            Some(file_name) => Some(dir.join(file_name)),
            None => match dir.parent() {
                // file_name == ".."
                Some(parent_dir) => Some(parent_dir.to_path_buf()),
                None => Some(PathBuf::from("/")), // Stop at the root.
            },
        },
        None => None,
    };
}

fn normalize_path(arg: &OsStr) -> OsString {
    let path = Path::new(arg);

    // Handle symlinks.
    if let Ok(metadata) = path.symlink_metadata() {
        if metadata.file_type().is_symlink() {
            return match symlink_canonicalize(path) {
                Some(normalized_path) => normalized_path.into_os_string(),
                None => OsString::from(arg),
            };
        }
    }

    // Handle normal files.
    match path.canonicalize() {
        Ok(normalized_pathname) => normalized_pathname.into_os_string(),
        Err(_) => OsString::from(arg),
    }
}

fn filter_arguments(
    args: impl Iterator<Item = OsString>,
    protected_paths: &HashSet<PathBuf>,
) -> Vec<OsString> {
    args.filter(|arg| {
        if protected_paths.contains(&PathBuf::from(normalize_path(arg))) {
            println!("safe-rm: Skipping {}.", arg.to_string_lossy());
            false
        } else {
            true
        }
    })
    .collect()
}

fn read_config_files(globals: &[&str], locals: &[&str]) -> HashSet<PathBuf> {
    let mut protected_paths: HashSet<PathBuf> =
        globals.iter().filter_map(read_config).flatten().collect();

    if let Ok(value) = std::env::var("HOME") {
        let home_dir = Path::new(&value);
        protected_paths.extend(
            locals
                .iter()
                .filter_map(|f| read_config(&home_dir.join(Path::new(f))))
                .flatten(),
        );
    }

    protected_paths.extend(DEFAULT_PATHS.iter().map(PathBuf::from));

    protected_paths
}

fn run(
    rm_binary: &OsStr,
    args: impl Iterator<Item = OsString>,
    globals: &[&str],
    locals: &[&str],
) -> i32 {
    let protected_paths = read_config_files(globals, locals);
    let filtered_args = filter_arguments(args, &protected_paths);

    // Run the real rm command, returning with the same error code.
    match process::Command::new(rm_binary)
        .args(&filtered_args)
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => {
            println!(
                "safe-rm: Failed to run the {} command.",
                rm_binary.to_string_lossy()
            );
            1
        }
    }
}

fn ensure_real_rm_is_callable(real_rm: &OsStr) -> io::Result<()> {
    // Make sure we're not calling ourselves recursively.
    if fs::canonicalize(real_rm)? == fs::canonicalize(std::env::current_exe()?)? {
        return Err(Error::new(
            ErrorKind::Other,
            "The real rm command points to the safe-rm binary.",
        ));
    }
    Ok(())
}

fn main() {
    if let Err(e) = ensure_real_rm_is_callable(REAL_RM.as_ref()) {
        println!(
            "safe-rm: Cannot check that the real \"rm\" binary is callable: {}",
            e
        );
        process::exit(1);
    }
    process::exit(run(
        REAL_RM.as_ref(),
        std::env::args_os().skip(1),
        &[GLOBAL_CONFIG, LOCAL_GLOBAL_CONFIG],
        &[USER_CONFIG, LEGACY_USER_CONFIG],
    ));
}
