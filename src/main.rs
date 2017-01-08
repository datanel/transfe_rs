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

#[derive(RustcDecodable, Debug)]
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
        let lambda2 = self.stop_lon.to_radians();

        let x = f64::sin((phi2 - phi1) / 2.).powi(2);
        let y = f64::cos(phi1) * f64::cos(phi2) * f64::sin((lambda2 - lambda1) / 2.).powi(2);

        2. * EARTH_RADIUS * f64::asin(f64::sqrt(x + y))
    }
}

struct StopPointIter<'a, R: std::io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    stop_id_pos: usize,
    stop_lat_pos: usize,
    stop_lon_pos: usize,
    location_type_pos: Option<usize>,
}
impl<'a, R: std::io::Read + 'a> StopPointIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>) -> Option<Self> {
        let headers = if let Ok(hs) = r.headers() { hs } else { return None; };
        let get = |name| headers.iter().position(|s| s == name);
        let stop_id_pos = if let Some(pos) = get("stop_id") { pos } else { return None; };
        let stop_lat_pos = if let Some(pos) = get("stop_lat") { pos } else { return None; };
        let stop_lon_pos = if let Some(pos) = get("stop_lon") { pos } else { return None; };
        Some(StopPointIter {
            iter: r.records(),
            stop_id_pos: stop_id_pos,
            stop_lat_pos: stop_lat_pos,
            stop_lon_pos: stop_lon_pos,
            location_type_pos: get("location_type"),
        })
    }
    fn get_location_type(&self, record: &[String]) -> Option<u8> {
        self.location_type_pos.and_then(|pos| record.get(pos).and_then(|s| s.parse().ok()))
    }
}
impl<'a, R: std::io::Read + 'a> Iterator for StopPointIter<'a, R> {
    type Item = csv::Result<StopPoint>;
    fn next(&mut self) -> Option<Self::Item> {
        fn get(record: &[String], pos: usize) -> csv::Result<&str> {
            match record.get(pos) {
                Some(s) => Ok(s),
                None => Err(csv::Error::Decode(format!("Failed accessing record '{}'.", pos)))
            }
        }
        fn parse_f64(s: &str) -> csv::Result<f64> {
            s.parse().map_err(|_| {
                csv::Error::Decode(format!("Failed converting '{}' from str.", s))
            })
        }

        self.iter.next().map(|r| r.and_then(|r| {
            let stop_id = try!(get(&r, self.stop_id_pos));
            let stop_lat = try!(get(&r, self.stop_lat_pos));
            let stop_lat = try!(parse_f64(stop_lat));
            let stop_lon = try!(get(&r, self.stop_lon_pos));
            let stop_lon = try!(parse_f64(stop_lon));
            Ok(StopPoint {
                stop_id: stop_id.to_string(),
                stop_lat: stop_lat,
                stop_lon: stop_lon,
                location_type: self.get_location_type(&r),
            })
        }))
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

    let stop_point_list: Vec<StopPoint> = StopPointIter::new(&mut rdr)
        .expect("Can't find needed fields in the header.")
        .filter_map(|rc| {
            rc.map_err(|e| {
                    println!("error at csv line decoding : {}", e)
                })
                .ok()
        })
        .filter(|st: &StopPoint| st.location_type.unwrap_or(0) == 0)
        .collect();

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
