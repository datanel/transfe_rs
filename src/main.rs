extern crate csv;
extern crate serde;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::error::Error;
use structopt::StructOpt;

const EARTH_RADIUS: f64 = 6372797.560856;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i", help = "GTFS stops.txt file")]
    input: String,

    #[structopt(long = "output", short = "o", default_value = "transfers.txt",
                help = "GTFS transfers.txt file")]
    output: String,

    #[structopt(long = "max-distance", short = "d", default_value = "500",
                help = "The max distance in meters to compute the tranfer")]
    max_distance: f64,

    #[structopt(long = "walking-speed", short = "s", default_value = "0.785",
                help = "The walking speed in meters per second. \
                        You may want to divide your initial speed by \
                        sqrt(2) to simulate Manhattan distances")]
    walking_speed: f64,

    #[structopt(long = "transfer-time", short = "t", default_value = "0",
                help = "Transfer time in second.")]
    transfer_time: u32,
}

#[derive(Debug,Deserialize)]
struct StopPoint {
    stop_id: String,
    stop_lat: f64,
    stop_lon: f64,
    location_type: Option<u8>,
}

impl StopPoint {
    fn distance_to(&self, other: &StopPoint) -> f64 {
        let phi1 = self.stop_lat.to_radians();
        let phi2 = other.stop_lat.to_radians();
        let lambda1 = self.stop_lon.to_radians();
        let lambda2 = other.stop_lon.to_radians();

        let x = f64::sin((phi2 - phi1) / 2.).powi(2);
        let y = f64::cos(phi1) * f64::cos(phi2) * f64::sin((lambda2 - lambda1) / 2.).powi(2);

        2. * EARTH_RADIUS * f64::asin(f64::sqrt(x + y))
    }
}

macro_rules! fatal(
    ($($arg:tt)*) => { {
        use std::io::Write;
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
        ::std::process::exit(1)
    } }
);

fn get_stop_points<P: AsRef<Path>>(path: P) -> Result<Vec<StopPoint>, Box<Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let stop_point_list: Vec<StopPoint> = rdr.deserialize()
        .filter_map(|rc| {
                        rc.map_err(|e| println!("error at csv line decoding : {}", e))
                            .ok()
                    })
        .filter(|st: &StopPoint| st.location_type.unwrap_or(0) == 0)
        .collect();

    Ok(stop_point_list)
}

fn main() {
    let args = Args::from_args();
    match get_stop_points(args.input) {
        Err(err) => fatal!("Error: {}", err),
        Ok(stop_point_list) => {
            let mut wtr = csv::Writer::from_path(args.output).unwrap();
            wtr.write_record(&["from_stop_id", "to_stop_id", "transfer_type", "min_transfer_time"])
                .unwrap();

            for stop_point_1 in &stop_point_list {
                for stop_point_2 in &stop_point_list {
                    let distance = stop_point_1.distance_to(stop_point_2);
                    let min_transfer_time = (distance / args.walking_speed) as u32 +
                                            args.transfer_time;
                    if distance <= args.max_distance {
                        wtr.write_record(&[&stop_point_1.stop_id,
                                            &stop_point_2.stop_id,
                                            "2",
                                            &min_transfer_time.to_string()])
                            .unwrap();
                    }
                }
            }
        }
    }
}
