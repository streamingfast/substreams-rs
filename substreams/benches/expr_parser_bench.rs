//! Benchmarks for the expression parser
//!
//! Run with: cargo bench --package substreams
//!
//! Compares Pest-based parser (expr_parser) vs experimental sqe parser.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use substreams::expr_parser::ExprMatcher as PestMatcher;
use substreams::sqe::ExprMatcher as SqeMatcher;

/// Representative expressions for real-world use cases
static EXPRESSIONS: &[(&str, &str)] = &[
    // Single address/key - most common case
    ("single_key", "transfer"),
    // Simple expression with 2-3 keys
    ("simple_or", "transfer || approval"),
    ("simple_and", "pool_address && swap"),
    // Medium complexity - nested with a few keys
    ("medium", "(transfer || approval) && token_address"),
    // Larger expression with many addresses (10+ keys)
    (
        "large",
        "addr1 || addr2 || addr3 || addr4 || addr5 || addr6 || addr7 || addr8 || addr9 || addr10 || addr11 || addr12",
    ),
];

/// Realistic key sets to match against
static SMALL_KEYS: &[&str] = &["transfer", "pool_address", "swap"];
static MEDIUM_KEYS: &[&str] = &[
    "transfer",
    "approval",
    "token_address",
    "pool_address",
    "swap",
];
static LARGE_KEYS: &[&str] = &[
    "addr1", "addr2", "addr3", "addr4", "addr5", "addr6", "addr7", "addr8", "addr9", "addr10",
    "other1", "other2", "other3",
];

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    for (name, expr) in EXPRESSIONS {
        group.bench_with_input(BenchmarkId::new("pest", name), expr, |b, expr| {
            b.iter(|| PestMatcher::new(black_box(expr)).unwrap());
        });
        group.bench_with_input(BenchmarkId::new("sqe", name), expr, |b, expr| {
            b.iter(|| SqeMatcher::parse(black_box(expr)).unwrap());
        });
    }

    group.finish();
}

fn bench_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching");

    // Match simple expression against small key set
    let pest_simple = PestMatcher::new("transfer || approval").unwrap();
    let sqe_simple = SqeMatcher::parse("transfer || approval").unwrap();
    group.bench_function(BenchmarkId::new("pest", "simple"), |b| {
        b.iter(|| pest_simple.matches_keys(black_box(SMALL_KEYS)));
    });
    group.bench_function(BenchmarkId::new("sqe", "simple"), |b| {
        b.iter(|| sqe_simple.matches_keys(black_box(SMALL_KEYS)));
    });

    // Match medium expression against medium key set
    let pest_medium = PestMatcher::new("(transfer || approval) && token_address").unwrap();
    let sqe_medium = SqeMatcher::parse("(transfer || approval) && token_address").unwrap();
    group.bench_function(BenchmarkId::new("pest", "medium"), |b| {
        b.iter(|| pest_medium.matches_keys(black_box(MEDIUM_KEYS)));
    });
    group.bench_function(BenchmarkId::new("sqe", "medium"), |b| {
        b.iter(|| sqe_medium.matches_keys(black_box(MEDIUM_KEYS)));
    });

    // Match large expression against large key set
    let large_expr =
        "addr1 || addr2 || addr3 || addr4 || addr5 || addr6 || addr7 || addr8 || addr9 || addr10";
    let pest_large = PestMatcher::new(large_expr).unwrap();
    let sqe_large = SqeMatcher::parse(large_expr).unwrap();
    group.bench_function(BenchmarkId::new("pest", "large"), |b| {
        b.iter(|| pest_large.matches_keys(black_box(LARGE_KEYS)));
    });
    group.bench_function(BenchmarkId::new("sqe", "large"), |b| {
        b.iter(|| sqe_large.matches_keys(black_box(LARGE_KEYS)));
    });

    group.finish();
}

/// Repeated matching - the main real-world use case
/// Same expression matched against many different key sets
fn bench_repeated_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_matching");

    let key_sets: Vec<&[&str]> = vec![
        &["transfer"],
        &["approval", "token_address"],
        &["swap", "pool_address", "amount"],
        &["unknown1", "unknown2"],
        &["transfer", "approval", "token_address", "pool_address"],
    ];

    let expr = "(transfer || approval) && (token_address || pool_address)";
    let pest_matcher = PestMatcher::new(expr).unwrap();
    let sqe_matcher = SqeMatcher::parse(expr).unwrap();

    group.bench_function("pest", |b| {
        b.iter(|| {
            for keys in &key_sets {
                black_box(pest_matcher.matches_keys(black_box(*keys)));
            }
        });
    });

    group.bench_function("sqe", |b| {
        b.iter(|| {
            for keys in &key_sets {
                black_box(sqe_matcher.matches_keys(black_box(*keys)));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_matching,
    bench_repeated_matching
);
criterion_main!(benches);
