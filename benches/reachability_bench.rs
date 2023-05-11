use criterion::{criterion_group, criterion_main, Criterion};
use reveaal::tests::refinement::Helper::json_run_query;
pub mod flamegraph;
use flamegraph::flamegraph_profiler::FlamegraphProfiler;

static PATH: &str = "samples/json/EcdarUniversity";

fn bench_reachability(c: &mut Criterion, query: &str) {
    c.bench_function(query, |b| b.iter(|| json_run_query(PATH, query)));
}

fn reachability_benchmarking(c: &mut Criterion) {
    bench_reachability(
        c,
        "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9",
    );
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher @ Administration.L3 && Machine.L5 && Researcher.L9 -> Administration.L0 && Machine.L5 && Researcher.U0",
    );
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher @ Administration.L0 && Machine.L5 && Researcher.U0 -> Administration.L3 && Machine.L5 && Researcher.L9",
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 && Machine.y<6 -> Machine.L4 && Machine.y<=6",
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 -> Machine.L4 && Machine.y>7",
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L4 && Machine.y<=6 -> Machine.L5 && Machine.y>=4",
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 && Machine.y<1 -> Machine.L5 && Machine.y<2",
    );
    bench_reachability(c, "reachability: Machine @ Machine.L5 -> Machine.L5");
    bench_reachability(
        c,
        "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L5 && Researcher.L7",
    );
    bench_reachability(
        c,
        "reachability: Researcher @ Researcher.U0 -> Researcher.L7",
    );
}

criterion_group! {
  name = reachability_benches;
  config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
  targets = reachability_benchmarking
}

criterion_main!(reachability_benches);
