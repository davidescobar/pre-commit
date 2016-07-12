extern crate regex;

use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};


enum ProjectType { Rails, Phoenix }


fn main() {
    let unit_test_stdout =
        match get_project_type() {
            Some(ProjectType::Rails) => {
                println!("Running RSpec. This may take a minute.....");
                Command::new("bin/rake").arg("spec").output().unwrap().stdout
            },
            Some(ProjectType::Phoenix) => {
                println!("Running ExUnit. This may take a minute.....");
                Command::new("mix").arg("test").output().unwrap().stdout
            }
            _ => {
                println!("\nThe project type for the unit tests was not recognized.");
                println!("At this time, only Ruby on Rails and Elixir projects are recognized.\n");
                Vec::<u8>::new()
            }
        };
    if unit_test_stdout.is_empty() { return; }

    let output_lines = String::from_utf8(unit_test_stdout).unwrap_or("".to_owned())
                                                          .lines()
                                                          .map(str::to_owned)
                                                          .collect::<Vec<String>>();
    for line in &output_lines { println!("{}", line); }
    let zero_failures_re = Regex::new(r"(?i)0 failures").unwrap();
    let tests_passed = output_lines.iter().any(|line| zero_failures_re.is_match(line));
    if tests_passed {
        println!("\nAll tests passed! Committing...\n");
        process::exit(0);
    } else {
        println!("\n1 or more test failures detected. Commit cancelled.");
        println!("Please fix the failing tests and try your commit again.\n");
        process::exit(1);
    }
}


fn find_project_root_path() -> Option<String> {
    env::current_dir().ok().and_then(|cwd| cwd.to_str().map(str::to_owned))
}


fn get_project_type() -> Option<ProjectType> {
    find_project_root_path().and_then(|project_root_path|
        if let Ok(dir_entries_iter) = fs::read_dir(Path::new(&project_root_path)) {
            let files =
                dir_entries_iter.filter_map(|dir_entry_result| {
                    dir_entry_result.ok().and_then(|dir_entry|
                        if dir_entry.path().is_file() {
                            dir_entry.path().file_name().and_then(|path_os_str|
                                path_os_str.to_str().map(str::to_owned)
                            )
                        } else {
                            None
                        }
                    )
                }).collect::<Vec<String>>();

            for file in &files {
                let normalized_file = file.to_lowercase();
                if normalized_file.ends_with("gemfile") {
                    return Some(ProjectType::Rails)
                } else if normalized_file.ends_with("mix.exs") {
                    return Some(ProjectType::Phoenix);
                }
            }
            None
        } else {
            None
        }
    )
}
