use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box as bb;

fn generate_haystack() -> Vec<String> {
    let mut haystack = Vec::with_capacity(5000);
    for i in 0..5000 {
        haystack.push(format!("Artist {} - Album Title {} (2020)", i % 100, i));
    }
    // inject some known good matches
    haystack.push("fleet foxes - shore (2020)".to_string());
    haystack.push("mac demarco - salad days (2014)".to_string());
    haystack.push("xyz band - xyz album (2023)".to_string());
    haystack
}

fn bench_matchers(c: &mut Criterion) {
    let haystack = generate_haystack();
    let queries = vec!["fleet foxes", "shore", "mac", "xyz"];

    let mut group = c.benchmark_group("fuzzy_matchers_5k");

    for query in &queries {
        group.bench_with_input(BenchmarkId::new("frizbee", query), query, |b, q| {
            let config = frizbee::Config::default();
            let mut matcher = frizbee::Matcher::new(*q, &config);
            b.iter(|| {
                let matches: Vec<_> = matcher.match_iter(haystack.iter()).collect();
                bb(matches)
            });
        });

        group.bench_with_input(BenchmarkId::new("nucleo_raw", query), query, |b, q| {
            let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
            let pattern = nucleo::pattern::Pattern::parse(
                q,
                nucleo::pattern::CaseMatching::Ignore,
                nucleo::pattern::Normalization::Smart,
            );
            b.iter(|| {
                let mut matches = Vec::new();
                let mut buf = Vec::new();
                for item in &haystack {
                    let str_utf32 = nucleo::Utf32Str::new(item, &mut buf);
                    let score = pattern.score(str_utf32, &mut matcher);
                    if score.is_some() {
                        matches.push((score, item));
                    }
                }
                bb(matches)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_matchers);
criterion_main!(benches);
