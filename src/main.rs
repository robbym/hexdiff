#![allow(dead_code)]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use itertools::Itertools;
use serde_json;
use structopt::StructOpt;

mod ihex16;
use ihex16::IHex16File;

mod diff;
use diff::{IHex16Diff, IHex16DiffEngine};

#[derive(StructOpt, Debug)]
#[structopt(name = "hexdiff", author = "Robby Madruga <robbymadruga@gmail.com>")]
struct Opt {
    #[structopt(name = "FILE_1", parse(from_os_str))]
    file_1: PathBuf,

    #[structopt(name = "FILE_2", parse(from_os_str))]
    file_2: PathBuf,

    /// Optionally output diff to a file instead of stdout
    #[structopt(short, long, parse(from_os_str), value_names = &["file"])]
    output: Option<PathBuf>,

    /// Print non-differences as well
    #[structopt(short, long)]
    all: bool,

    /// Format output as JSON
    #[structopt(short, long)]
    json: bool,

    /// Specify an address or address range to ignore{n}
    /// Omitting a bound on a range defaults it to the min or max respectfully{n}
    /// Examples:{n}
    /// -i 4000{n}
    /// -i 4000:6000{n}
    /// -i 4000:{n}
    /// -i :6000
    #[structopt(short, long, value_names = &["address | range"])]
    ignore: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    let hex_1 = IHex16File::from_reader(&mut File::open(opt.file_1).unwrap());
    let hex_2 = IHex16File::from_reader(&mut File::open(opt.file_2).unwrap());

    let diff_iter = IHex16DiffEngine::diff(hex_1, hex_2);
    let diff_list;

    let ignore_list = opt
        .ignore
        .iter()
        .map(|range| {
            match range
                .split(":")
                .map(|x| u32::from_str_radix(x, 16).ok())
                .collect::<Vec<Option<u32>>>()
                .as_slice()
            {
                [Some(a), Some(b)] => *a..=*b,
                [None, Some(b)] => 0..=*b,
                [Some(a), None] => *a..=core::u32::MAX,
                [Some(a)] => *a..=*a,
                _ => panic!("INVALID IGNORE RANGE"),
            }
        })
        .collect::<Vec<_>>();

    if !opt.all {
        diff_list = diff_iter
            .filter(|x| !ignore_list.iter().any(|y| IHex16Diff::in_range(x, y)))
            .filter(IHex16Diff::is_diff)
            .collect::<Vec<_>>();
    } else {
        diff_list = diff_iter.collect::<Vec<_>>();
    }

    let output: String;

    if opt.json {
        output = serde_json::to_string_pretty(&diff_list).unwrap();
    } else {
        output = diff_list.iter().join("\n");
    }

    if let Some(output_file_path) = opt.output {
        let mut output_file = File::create(output_file_path).unwrap();
        output_file.write(output.as_bytes()).unwrap();
    } else {
        println!("{}", output);
    }
}
