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

use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
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

fn read_config<P: AsRef<Path>>(filename: P, paths: &mut Vec<String>) {
    if !filename.as_ref().exists() {
        return ();
    }
    match File::open(&filename) {
        Ok(f) => {
            let reader = io::BufReader::new(f);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        paths.push(line);
                    },
                    Err(_) => {
                        println!("Invalid line found in {} and ignored.", filename.as_ref().display());
                    }
                }
            }
        },
        Err(_) => {
            println!("Could not open configuration file: {}", filename.as_ref().display());
            ()
        }
    }
}

fn normalize_path(pathname: &str) -> String {
    match fs::canonicalize(pathname) {
        Ok(normalized_pathname) => {
            match normalized_pathname.to_str() {
                Some(normalized_pathname_str) => {
                    normalized_pathname_str.to_string()
                },
                None => pathname.to_string()
            }
        },
        Err(_) => pathname.to_string()
    }
}

#[test]
fn test_normalize_path() {
    assert_eq!(normalize_path("/"), "/");
    assert_eq!(normalize_path("/../."), "/");
    assert_eq!(normalize_path("/usr"), "/usr");
    assert_eq!(normalize_path("/usr/"), "/usr");
    assert_eq!(normalize_path("/home/../usr"), "/usr");
    assert_eq!(normalize_path(""), "");
    assert_eq!(normalize_path("foo"), "foo");
    assert_eq!(normalize_path("/tmp/�/"), "/tmp/�/");
}

fn filter_pathnames(args: impl Iterator<Item = String>, protected_paths: &Vec<String>) -> Vec<String> {
    let mut filtered_args = Vec::new();
    for pathname in args {
        let mut is_symlink = false;
        match fs::symlink_metadata(&pathname) {
            Ok(metadata) => {
                is_symlink = metadata.file_type().is_symlink();
            },
            Err(_) => ()
        }

        let normalized_pathname = normalize_path(&pathname);
        println!("{} -> {}", pathname, normalized_pathname); // TODO: remove this line
        if protected_paths.contains(&normalized_pathname) && !is_symlink {
            println!("safe-rm: skipping {}", pathname);
        } else {
            filtered_args.push(pathname);
        }
    }
    filtered_args
}

#[test]
fn test_filter_pathnames() {
    // Simple cases
    assert_eq!(filter_pathnames(vec!["/safe".to_string()].into_iter(),
                                &vec!["/safe".to_string()]),
               Vec::<String>::new());
    assert_eq!(filter_pathnames(vec!["/safe".to_string(), "/unsafe".to_string()].into_iter(),
                                &vec!["/safe".to_string()]),
               vec!["/unsafe".to_string()]);

    // Degenerate cases
    assert_eq!(filter_pathnames(Vec::<String>::new().into_iter(),
                                &Vec::<String>::new()),
               Vec::<String>::new());
    assert_eq!(filter_pathnames(vec!["/safe".to_string(), "/unsafe".to_string()].into_iter(),
                                &Vec::<String>::new()),
               vec!["/safe".to_string(), "/unsafe".to_string()]);
    assert_eq!(filter_pathnames(Vec::<String>::new().into_iter(),
                                &vec!["/safe".to_string()]),
               Vec::<String>::new());

    // Relative path
    assert_eq!(filter_pathnames(vec!["/../".to_string(), "/unsafe".to_string()].into_iter(),
                                &vec!["/".to_string()]),
               vec!["/unsafe".to_string()]);
}

fn finalize_protected_paths(protected_paths: &mut Vec<String>) {
    if protected_paths.is_empty() {
        for path in DEFAULT_PATHS {
            protected_paths.push(path.to_string());
        }
    }
    protected_paths.sort();
    protected_paths.dedup();
    println!("{:#?}", protected_paths);  // TODO: remove this line
}

#[test]
fn test_finalize_protected_paths() {
    {
        let mut paths = vec![];
        finalize_protected_paths(&mut paths);
        assert_eq!(paths, DEFAULT_PATHS);
    }
    {
        let mut paths = vec!["/two".to_string(), "/one".to_string()];
        finalize_protected_paths(&mut paths);
        assert_eq!(paths, vec!["/one".to_string(), "/two".to_string()]);
    }
    {
        let mut paths = vec!["/one".to_string(), "/one".to_string()];
        finalize_protected_paths(&mut paths);
        assert_eq!(paths, vec!["/one".to_string()]);
    }
}

fn read_config_files() -> Vec<String> {
    let mut protected_paths = Vec::new();
    read_config(GLOBAL_CONFIG, &mut protected_paths);
    read_config(LOCAL_GLOBAL_CONFIG, &mut protected_paths);
    match std::env::var("HOME") {
        Ok(value) => {
            let home_dir = Path::new(&value);
            read_config(&home_dir.join(Path::new(USER_CONFIG)), &mut protected_paths);
            read_config(&home_dir.join(Path::new(LEGACY_USER_CONFIG)), &mut protected_paths);
        },
        Err(_) => ()
    }
    protected_paths
}

fn main() {
    // Make sure we're not calling ourselves recursively.
    if fs::canonicalize(REAL_RM).unwrap() == fs::canonicalize(std::env::current_exe().unwrap()).unwrap() {
        println!("safe-rm cannot find the real \"rm\" binary");
        process::exit(1);
    }

    let mut protected_paths = read_config_files();
    finalize_protected_paths(&mut protected_paths);

    let filtered_args = filter_pathnames(std::env::args().skip(1), &protected_paths);

    println!("{} {:#?}", REAL_RM, filtered_args);  // TODO: remove this line
    // TODO: Run the real rm command, returning with the same error code
}