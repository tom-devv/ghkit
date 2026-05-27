use divan::{Bencher, black_box};
use ghkit::git::client::GhRepo;

#[divan::bench]
fn bench_get_all_commits() {
    let repo = GhRepo::open(black_box("../ghexample")).unwrap();
    let _commits = black_box(repo.get_all_commits().unwrap());
}

#[divan::bench(sample_count = 10, sample_size = 1, max_time = 5.0)]
fn bench_get_all_contrib(bencher: Bencher) {
    let repo = GhRepo::open(black_box("../ghexample")).unwrap();
    bencher.bench_local(|| {
        let _contribs = divan::black_box(repo.get_entire_repo_contribution().unwrap());
    });
}

fn main() {
    divan::main();
}
