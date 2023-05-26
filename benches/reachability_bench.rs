use criterion::{criterion_group, criterion_main, Criterion};
pub mod flamegraph;
use flamegraph::flamegraph_profiler::FlamegraphProfiler;
use reveaal::extract_system_rep::create_executable_query;
use reveaal::tests::TEST_SETTINGS;
use reveaal::{parse_queries, ComponentLoader, JsonProjectLoader, Query};

const PATH: &str = "samples/json/EcdarUniversity";
static mut LOADER: Option<Box<dyn ComponentLoader>> = None;

fn bench_reachability(c: &mut Criterion, query: &str) {
    c.bench_function(query, |b| {
        b.iter(|| {
            let query = parse_queries::parse_to_expression_tree(query)
                .unwrap()
                .remove(0);
            let q = Query {
                query: Option::from(query),
                comment: "".to_string(),
            };

            let query =
                create_executable_query(&q, unsafe { &mut **LOADER.as_mut().unwrap() }).unwrap();

            query.execute()
        })
    });
}

fn reachability_benchmarking(c: &mut Criterion) {
    let mut loader =
        JsonProjectLoader::new_loader(PATH.to_string(), TEST_SETTINGS).to_comp_loader();
    let _ = vec![
        loader.get_component("Machine").clone(),
        loader.get_component("Researcher").clone(),
        loader.get_component("Administration").clone(),
    ];
    unsafe { LOADER = Some(loader) };
    bench_reachability(
        c,
        "reachability: Machine || Researcher -> [L5, L6](); [L4, L9]()",
    );
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher -> [L3, L5, L9](); [L0, L5, U0]()",
    );
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher -> [L0, L5, U0](); [L3, L5, L9]()",
    );
    bench_reachability(c, "reachability: Machine -> [L5](y<6); [L4](y<=6)");
    bench_reachability(c, "reachability: Machine -> [L5](); [L4](y>7)");
    bench_reachability(c, "reachability: Machine -> [L4](y<=6); [L5](y>=4)");
    bench_reachability(c, "reachability: Machine -> [L5](y<1); [L5](y<2)");
    bench_reachability(c, "reachability: Machine -> [L5](); [L5]()");
    bench_reachability(
        c,
        "reachability: Machine || Researcher -> [L5, U0](); [L5, L7]()",
    );
    bench_reachability(c, "reachability: Researcher -> [U0](); [L7]()");
}

criterion_group! {
  name = reachability_benches;
  config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
  targets = reachability_benchmarking
}

criterion_main!(reachability_benches);
