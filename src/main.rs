extern crate rustc_serialize;
extern crate docopt;
extern crate csv;

use docopt::Docopt;

const EARTH_RADIUS: f64 = 6372797.560856;

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

#[derive(RustcDecodable)]
struct StopPoint {
    stop_id: String,
    // stop_code: Option<String>,
    _stop_name: String,
    _stop_desc: Option<String>,
    stop_lat: f64,
    stop_lon: f64,
    _zone_id: Option<String>,
    _stop_url: Option<String>,
    location_type: Option<u8>,
    _parent_station: Option<u8>,
    // stop_timezone: Option<String>,
    _wheelchair_boarding: Option<u8>,
}

impl StopPoint {
    fn distance_to(&self, other: &StopPoint) -> f64 {
        let phi1 = self.stop_lat.to_radians();
        let phi2 = other.stop_lat.to_radians();
        let lambda1 = self.stop_lon.to_radians();
        let lambda2 = self.stop_lon.to_radians();


        let x = f64::sin((phi2 - phi1) / 2.).powi(2);
        let y = f64::cos(phi1) * f64::cos(phi2) * f64::sin((lambda2 - lambda1) / 2.).powi(2);

        2. * EARTH_RADIUS * f64::asin(f64::sqrt(x + y))
    }
}

fn main() {
    println!("Launching transfe_rs...");

    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.decode())
        .unwrap_or_else(|e| e.exit());

    let mut rdr = csv::Reader::from_file(args.flag_input)
        .unwrap()
        .double_quote(true);

    let mut stop_point_list = vec![];
    for record in rdr.decode() {
        let stop_point: StopPoint = record.unwrap();
        if stop_point.location_type.unwrap_or(0) == 0 {
            stop_point_list.push(stop_point)
        }
    }

    let mut wtr = csv::Writer::from_file(args.flag_output).unwrap();
    wtr.encode(("from_stop_id", "to_stop_id", "transfer_type", "min_transfer_time"))
        .unwrap();

    for stop_point_1 in &stop_point_list {
        for stop_point_2 in &stop_point_list {
            let distance = stop_point_1.distance_to(stop_point_2);
            if stop_point_1.distance_to(stop_point_2) <= 500. {
                wtr.encode((&stop_point_1.stop_id, &stop_point_2.stop_id, 2, distance / 1.11))
                        .unwrap();
            }
        }
    }
}
