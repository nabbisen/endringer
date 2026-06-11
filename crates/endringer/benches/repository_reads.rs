//! Criterion benchmarks for endringer repository read operations (RFC 017).
//!
//! ## Running
//!
//! ```sh
//! cargo bench -p endringer
//! ```
//!
//! ## Performance classification
//!
//! | Class | Meaning |
//! |---|---|
//! | Cheap | Suitable for frequent status-widget refresh (< 1 ms on typical repos) |
//! | Moderate | Suitable for user-triggered UI refresh (< 50 ms on typical repos) |
//! | Expensive | Should be bounded, paged, or triggered intentionally |
//!
//! See `docs/src/development/performance.md` for recorded baselines.

use criterion::{criterion_group, criterion_main, Criterion};
use endringer::repository::repository;
use endringer::{CommitQuery, SortOrder};
use std::path::PathBuf;
use std::process::Command;

// ── Fixture builder ───────────────────────────────────────────────────────── //

fn make_bench_repo(commits: usize, files_per_commit: usize) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let p = dir.path();

    for cmd in [
        vec!["init", "-b", "main"],
        vec!["config", "user.email", "bench@local"],
        vec!["config", "user.name", "Bench"],
    ] {
        Command::new("git").args(&cmd).current_dir(p)
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_TERMINAL_PROMPT","0").stdin(std::process::Stdio::null())
            .status().expect("git");
    }

    for i in 0..commits {
        for f in 0..files_per_commit {
            let fname = format!("file_{i}_{f}.txt");
            std::fs::write(p.join(&fname), format!("commit {i} file {f}")).unwrap();
        }
        Command::new("git").args(["add","."]).current_dir(p)
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .status().expect("git add");
        Command::new("git").args(["commit","-m",&format!("commit {i}")]).current_dir(p)
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
            .stdin(std::process::Stdio::null()).status().expect("git commit");
    }
    dir
}

// ── Benchmark groups ──────────────────────────────────────────────────────── //

fn bench_status(c: &mut Criterion) {
    let dir = make_bench_repo(20, 5);
    let path: PathBuf = dir.path().to_path_buf();

    let mut group = c.benchmark_group("status");

    group.bench_function("status_digest", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.status_digest().unwrap());
    });

    group.bench_function("is_dirty", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.is_dirty().unwrap());
    });

    group.bench_function("worktree_status", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.worktree_status().unwrap());
    });

    group.finish();
    drop(dir); // keep alive until here
}

fn bench_refs(c: &mut Criterion) {
    let dir = make_bench_repo(5, 2);
    let path: PathBuf = dir.path().to_path_buf();
    // Add some tags.
    for i in 0..5 {
        Command::new("git").args(["tag",&format!("v0.{i}.0")])
            .current_dir(&path).env("GIT_CONFIG_NOSYSTEM","1")
            .env("GIT_CONFIG_GLOBAL","/dev/null").status().unwrap();
    }

    let mut group = c.benchmark_group("refs");

    group.bench_function("local_branches", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.local_branches().unwrap());
    });

    group.bench_function("list_tags_by_name", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.list_tags_sorted(SortOrder::ByName).unwrap());
    });

    group.bench_function("references", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.references().unwrap());
    });

    group.finish();
    drop(dir);
}

fn bench_history(c: &mut Criterion) {
    let dir = make_bench_repo(100, 2);
    let path: PathBuf = dir.path().to_path_buf();

    let mut group = c.benchmark_group("history");

    group.bench_function("list_commits_100", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.list_commits().unwrap());
    });

    group.bench_function("query_commits_page10", |b| {
        let repo = repository(&path).unwrap();
        b.iter(|| repo.query_commits(CommitQuery::head_page(10)).unwrap());
    });

    group.finish();
    drop(dir);
}

fn bench_object(c: &mut Criterion) {
    let dir = make_bench_repo(10, 3);
    let path: PathBuf = dir.path().to_path_buf();
    let head_id = {
        let repo = repository(&path).unwrap();
        repo.list_commits().unwrap()[0].commit_id.clone()
    };

    let mut group = c.benchmark_group("object");

    group.bench_function("tree_at_commit_root", |b| {
        let repo = repository(&path).unwrap();
        let id = head_id.clone();
        b.iter(|| repo.tree_at_commit(&id).unwrap());
    });

    group.bench_function("file_at_commit", |b| {
        let repo = repository(&path).unwrap();
        let id = head_id.clone();
        b.iter(|| repo.file_at_commit(std::path::Path::new("file_9_0.txt"), &id).unwrap());
    });

    group.finish();
    drop(dir);
}

criterion_group!(benches, bench_status, bench_refs, bench_history, bench_object);
criterion_main!(benches);
