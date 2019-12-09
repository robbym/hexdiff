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

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Print non-differences as well
    #[structopt(short, long)]
    all: bool,

    /// Format output as JSON
    #[structopt(short, long)]
    json: bool,
}

fn main() {
    let opt = Opt::from_args();

    let hex_1 = IHex16File::from_reader(&mut File::open(opt.file_1).unwrap());
    let hex_2 = IHex16File::from_reader(&mut File::open(opt.file_2).unwrap());

    let diff_iter = IHex16DiffEngine::diff(hex_1, hex_2);
    let diff_list;

    if !opt.all {
        diff_list = diff_iter.filter(IHex16Diff::is_diff).collect::<Vec<_>>();
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
