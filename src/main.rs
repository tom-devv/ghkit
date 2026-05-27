use std::path::PathBuf;

use gkit::{git::kit::GRepo, metrics::cadence::CadenceMetric};

fn main() {
    let repo_path = PathBuf::from("../ghexample");
    let repo = GRepo::open(&repo_path).unwrap();

    let cadence = CadenceMetric::full_report(&repo).unwrap();

    println!("{:?}", cadence);
    // match CadenceMetric::global(&repo) {
    //     Ok(per_second) => println!("{} commits per day", per_second * 60.0 * 60.0 * 24.0),
    //     Err(err) => eprintln!("{}", err),
    // }
    println!("Done!");
}
