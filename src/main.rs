use std::path::PathBuf;

use ghkit::git::client::GhRepo;

fn main() {
    let repo_path = PathBuf::from("../ghexample");
    let repo = GhRepo::open(&repo_path).unwrap();

    let _contribs = repo.get_entire_repo_contribution().unwrap();

    println!("Done!");
}
