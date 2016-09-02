extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
Building GTFS transfers.txt file from GTFS stops.txt.

Usage:
  transfe_rs --help
  transfe_rs --input=<file> [--output=<file>]

Options:
  -h, --help           Show this screen.
  -i, --input=<file>   GTFS stops.txt file.
  -o, --output=<file>  GTFS transfers.txt file [default: ./transfers.txt].
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_input: String,
    flag_output: String,
}

fn main() {
    println!("Launching transfe_rs...");

    let args: Args = Docopt::new(USAGE)
                         .and_then(|dopt| dopt.decode())
                         .unwrap_or_else(|e| e.exit());
    println!("input file: {:?}, output file {:?}",
             args.flag_input,
             args.flag_output);
}
