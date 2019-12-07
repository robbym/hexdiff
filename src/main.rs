use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;

mod ihex16;
use ihex16::IHex16File;

mod diff;
use diff::{IHex16Diff, IHex16DiffEngine};

#[derive(StructOpt, Debug)]
#[structopt(name = "hexdiff")]
struct Opt {
    #[structopt(name = "FILE_1", parse(from_os_str))]
    file_1: PathBuf,

    #[structopt(name = "FILE_2", parse(from_os_str))]
    file_2: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Print non-differences as well
    #[structopt(short, long)]
    all: bool
}

fn main() {
    let opt = Opt::from_args();

    let hex_1 = IHex16File::from_reader(&mut File::open(opt.file_1).unwrap());
    let hex_2 = IHex16File::from_reader(&mut File::open(opt.file_2).unwrap());

    let diff_list = IHex16DiffEngine::diff(hex_1, hex_2);

    let mut output = Vec::new();

    for diff in diff_list {
        match diff {
            IHex16Diff::Single {
                address,
                value_1,
                value_2,
            } if opt.all || value_1 != value_2 => {
                writeln!(
                    output,
                    "{:06X} {:06X} {:06X}",
                    address / 2,
                    value_1,
                    value_2
                )
                .unwrap();
            }

            IHex16Diff::Range {
                start,
                end,
                value_1,
                value_2,
            } if opt.all || value_1 != value_2 => {
                writeln!(
                    output,
                    "{:06X} {:06X} {:06X} {:06X}",
                    start / 2,
                    end / 2,
                    value_1,
                    value_2
                )
                .unwrap();
            }

            _ => {}
        }
    }

    if let Some(output_file) = opt.output {
        let mut diff_file = File::create(output_file).unwrap();
        diff_file.write_all(&mut output).unwrap();
    } else {
        unsafe {
            print!("{}", String::from_utf8_unchecked(output));
        }
    }
}
