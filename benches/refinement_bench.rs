use criterion::{criterion_group, criterion_main, Criterion};

pub mod flamegraph;

use flamegraph::flamegraph_profiler::FlamegraphProfiler;
use reveaal::extract_system_rep::create_executable_query;
use reveaal::tests::TEST_SETTINGS;
use reveaal::System::executable_query::ExecutableQuery;
use reveaal::System::query_failures::QueryResult;
use reveaal::{parse_queries, ComponentLoader, JsonProjectLoader, Query};

const PATH: &str = "samples/json/EcdarUniversity";

fn load_everything(loader: &mut Box<dyn ComponentLoader>) {
    let _ = loader.get_component("Adm2");
    let _ = loader.get_component("Administration");
    let _ = loader.get_component("HalfAdm1");
    let _ = loader.get_component("HalfAdm2");
    let _ = loader.get_component("Machine");
    let _ = loader.get_component("Machine2");
    let _ = loader.get_component("Machine3");
    let _ = loader.get_component("Machine4");
    let _ = loader.get_component("Researcher");
    let _ = loader.get_component("Spec");
}

fn construct_query<'a>(
    query: &str,
    loader: &'a mut Box<dyn ComponentLoader>,
) -> Box<dyn ExecutableQuery + 'a> {
    let query = parse_queries::parse_to_expression_tree(query)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    create_executable_query(&q, loader.as_mut()).unwrap()
}

fn bench_refinement(c: &mut Criterion, query: &str, loader: &mut Box<dyn ComponentLoader>) {
    c.bench_function(query, |b| {
        b.iter(|| match construct_query(query, loader).execute() {
            QueryResult::Refinement(Ok(_)) => assert!(true),
            _ => assert!(false),
        })
    });
}

fn bench_non_refinement(c: &mut Criterion, query: &str, loader: &mut Box<dyn ComponentLoader>) {
    c.bench_function(&format!("NOT {query}"), |b| {
        b.iter(|| match construct_query(query, loader).execute() {
            QueryResult::Refinement(Err(_)) => assert!(true),
            _ => assert!(false),
        })
    });
}

fn bench_self_refinement(c: &mut Criterion, query: &str, loader: &mut Box<dyn ComponentLoader>) {
    bench_refinement(c, &format!("refinement: {query} <= {query}"), loader);
}

fn self_refinement(c: &mut Criterion, loader: &mut Box<dyn ComponentLoader>) {
    bench_self_refinement(c, "Adm2", loader);
    bench_self_refinement(c, "Administration", loader);
    bench_self_refinement(c, "HalfAdm1", loader);
    bench_self_refinement(c, "HalfAdm2", loader);
    bench_self_refinement(c, "Machine", loader);
    bench_self_refinement(c, "Machine3", loader);
    bench_self_refinement(c, "Researcher", loader);
    bench_self_refinement(c, "Spec", loader);
    bench_self_refinement(c, "Administration || Researcher || Machine", loader);
}

fn refinement(c: &mut Criterion, loader: &mut Box<dyn ComponentLoader>) {
    bench_refinement(
        c,
        "refinement: Researcher <= Spec // Administration // Machine",
        loader,
    );
    bench_refinement(
        c,
        "refinement: Machine <= Spec // Administration // Researcher",
        loader,
    );
    bench_refinement(
        c,
        "refinement: Administration <= Spec // Researcher // Machine",
        loader,
    );
    bench_refinement(
        c,
        "refinement: Administration || Researcher <= Spec // Machine",
        loader,
    );
    bench_refinement(
        c,
        "refinement: Researcher || Machine <= Spec // Administration",
        loader,
    );
    bench_refinement(
        c,
        "refinement: Machine || Administration <= Spec // Researcher",
        loader,
    );
}

fn not_refinement(c: &mut Criterion, loader: &mut Box<dyn ComponentLoader>) {
    bench_non_refinement(
        c,
        "refinement: Adm2 <= Spec // Researcher // Machine",
        loader,
    );
    bench_non_refinement(
        c,
        "refinement: Machine <= Spec // Adm2 // Researcher",
        loader,
    );
    bench_non_refinement(
        c,
        "refinement: Adm2 || Researcher <= Spec // Machine",
        loader,
    );
}

fn all_refinements(c: &mut Criterion) {
    let mut loader =
        JsonProjectLoader::new_loader(PATH.to_string(), TEST_SETTINGS).to_comp_loader();
    load_everything(&mut loader);

    self_refinement(c, &mut loader);
    refinement(c, &mut loader);
    not_refinement(c, &mut loader);
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
    targets = all_refinements,
}

criterion_main!(benches);
