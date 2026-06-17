use git2::Repository;
use git_stats::config::Config;
use git_stats::output;
use git_stats::pipeline::run_scan;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn init_repo_with_commits(path: &Path, author: &str, email: &str, count: usize) {
    let repo = Repository::init(path).unwrap();
    let sig = git2::Signature::now(author, email).unwrap();
    let mut parent: Option<git2::Oid> = None;

    for i in 0..count {
        let mut index = repo.index().unwrap();
        let blob = repo.blob(format!("file {i}").as_bytes()).unwrap();
        let _ = index.add(&git2::IndexEntry {
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: 0,
            id: blob,
            flags: 0,
            flags_extended: 0,
            path: format!("f{i}.txt").into(),
        });
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let oid = if let Some(p) = parent {
            let pc = repo.find_commit(p).unwrap();
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("commit number {i} with some message"),
                &tree,
                &[&pc],
            )
            .unwrap()
        } else {
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("initial commit {i}"),
                &tree,
                &[],
            )
            .unwrap()
        };
        parent = Some(oid);
    }
}

#[test]
fn end_to_end_scan_fixture() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let repo_a = root.join("alpha");
    fs::create_dir_all(&repo_a).unwrap();
    init_repo_with_commits(&repo_a, "Alice", "alice@example.com", 2);

    let repo_b = root.join("beta");
    fs::create_dir_all(&repo_b).unwrap();
    init_repo_with_commits(&repo_b, "Bob", "bob@example.com", 1);

    let config = Config {
        scan_paths: vec![root.to_path_buf()],
        cache: git_stats::config::CacheConfig {
            enabled: false,
            ttl_hours: 1,
        },
        ..Config::default()
    };

    let output = run_scan(&config).unwrap();
    assert_eq!(output.stats.total_repos, 2);
    assert_eq!(output.stats.total_commits, 3);
    assert_eq!(output.stats.total_authors, 2);

    let json = serde_json::to_string(&output.stats).unwrap();
    assert!(json.contains("total_commits"));
}

#[test]
fn summary_output_format() {
    let stats = git_stats::stats::GlobalStats {
        total_repos: 3,
        total_commits: 10,
        total_authors: 2,
        stale_repos: 1,
        ..Default::default()
    };
    output::summary::print_summary(&stats);
}
