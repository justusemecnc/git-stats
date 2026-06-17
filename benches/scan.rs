use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_stats::config::Config;
use git_stats::pipeline::run_scan;
use git_stats::scanner::scan_repositories;
use tempfile::TempDir;

fn bench_scan_repos(c: &mut Criterion) {
    c.bench_function("scan_repositories_current_dir", |b| {
        let config = Config::default();
        b.iter(|| black_box(scan_repositories(&config).unwrap()));
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("bench-repo");
    std::fs::create_dir_all(&path).unwrap();
    let repo = git2::Repository::init(&path).unwrap();
    let sig = git2::Signature::now("Bench", "bench@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "bench commit",
        &repo.find_tree(tree_id).unwrap(),
        &[],
    )
    .unwrap();

    let config = Config {
        scan_paths: vec![tmp.path().to_path_buf()],
        cache: git_stats::config::CacheConfig {
            enabled: false,
            ttl_hours: 1,
        },
        ..Config::default()
    };

    c.bench_function("run_scan_small_fixture", |b| {
        b.iter(|| black_box(run_scan(&config).unwrap()));
    });
}

criterion_group!(benches, bench_scan_repos, bench_full_pipeline);
criterion_main!(benches);
