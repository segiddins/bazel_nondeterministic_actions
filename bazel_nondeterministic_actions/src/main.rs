use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::vec::Vec;
use std::string::String;
use std::time::{SystemTime};
use std::env;
use std::fs;
use std::path::PathBuf;

#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;

#[derive(Debug)]
enum LineType {
    ActionSeparator,
    Section(String),
    Mnemonic(String),
    Remotable(bool),
    Cacheable(bool),
    Diff,
    Boring,
}


fn parse_line(text: &str) -> LineType {
    lazy_static! {
        static ref DIFF: Regex = Regex::new("^[^ ]").unwrap();
        static ref SECTION: Regex = Regex::new("^  ([^ ]+) \\{").unwrap();
        static ref KV: Regex = Regex::new("^  ([^ ]+): (.+)").unwrap();
    }
    let separator: &str = "  ---------------------------------------------------------";

    if text == separator {
        return LineType::ActionSeparator;
    }

    if DIFF.is_match(text) {
        return LineType::Diff
    }

    if let Some(cap) = SECTION.captures(text) {
        return LineType::Section(cap[1].to_string());
    }

    if let Some(cap) = KV.captures(text) {
        match &cap[1] {
            "mnemonic" => return LineType::Mnemonic(cap[2].to_string()),
            "remotable" => return LineType::Remotable(cap[2].parse().unwrap()),
            "cacheable" => return LineType::Cacheable(cap[2].parse().unwrap()),
            &_ => {}
        }
    }

    LineType::Boring
}

fn print_summary(action_count: i64, line_count: usize, started: SystemTime) {
    let elapsed = started.elapsed().unwrap().as_secs_f64();
    println!("Processed {0:>5} total messages in {1:.1} seconds ({2:>7.0} messages/sec, {3:>10.0} lines/sec)", action_count, elapsed, action_count as f64 / elapsed, line_count as f64 / elapsed);
}


fn main() -> io::Result<()> {
    let started = SystemTime::now();
    let diff_path = fs::canonicalize(PathBuf::from(env::args().nth(1).unwrap())).unwrap();
    let file = File::open(&diff_path).unwrap();
    let reader = BufReader::new(file);

    let mut curr: Vec<String> = vec!();
    let mut action_count: i64 = 0;
    let mut line_count: usize = 0;

    let mut remotable: bool = false;
    let mut cacheable: bool = false;
    let mut section: Option<String> = None;
    let mut section_diffs: Vec<String> = vec!();
    let mut mnemonic: Option<String> = None;

    for (index, line) in reader.lines().enumerate() {
        line_count += 1;
        let line = line.unwrap();
        let line_type = parse_line(&line);
        match line_type {
            LineType::ActionSeparator => {
                action_count += 1;
                if action_count % 1_000 == 0 {
                    print_summary(action_count, index, started);
                }

                if !section_diffs.is_empty() && (remotable || cacheable) {
                    println!("{}", curr.join("\n"));
                }

                curr.clear();
                remotable = false;
                cacheable = false;
                section = None;
                section_diffs.clear();
                mnemonic = None;
            },
            LineType::Section(n) => section = Some(n),
            LineType::Remotable(r) => remotable = r,
            LineType::Cacheable(c) => cacheable = c,
            LineType::Mnemonic(m) => mnemonic = Some(m),
            LineType::Diff => {
                if let Some(s) = &section {
                    section_diffs.push(s.to_string())
                } else if !curr.is_empty() {
                    panic!("{:?}:{}: Diff outside of a section!", &diff_path, index + 1);
                }
            },
            LineType::Boring => {},
            // _ => panic!("Unhandled line {:#?}", line_type)
        }

        curr.push(line);
    }

    print_summary(action_count, line_count, started);

    Ok(())
}
