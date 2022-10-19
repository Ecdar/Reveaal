use std::vec;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reveaal::{
    tests::refinement::Helper::json_refinement_check,
    ProtobufServer::{
        services::{
            component::Rep, ecdar_backend_server::EcdarBackend, Component, ComponentsInfo,
            QueryRequest,
        },
        ConcreteEcdarBackend,
    },
};
use tonic::Request;

use criterion::async_executor::FuturesExecutor;

static PATH: &str = "samples/json/EcdarUniversity";

fn bench_refinement(c: &mut Criterion, query: &str) {
    c.bench_function(query, |b| {
        b.iter(|| {
            assert!(json_refinement_check(PATH, &format!("refinement: {query}")));
        })
    });
}

fn bench_non_refinement(c: &mut Criterion, query: &str) {
    c.bench_function(&format!("NOT {query}"), |b| {
        b.iter(|| {
            assert!(!json_refinement_check(
                PATH,
                &format!("refinement: {query}")
            ));
        })
    });
}

fn bench_self_refinement(c: &mut Criterion, query: &str) {
    bench_refinement(c, &format!("{query} <= {query}"));
}

fn self_refinement(c: &mut Criterion) {
    bench_self_refinement(c, "Adm2");
    bench_self_refinement(c, "Administration");
    bench_self_refinement(c, "HalfAdm1");
    bench_self_refinement(c, "HalfAdm2");
    bench_self_refinement(c, "Machine");
    bench_self_refinement(c, "Machine3");
    bench_self_refinement(c, "Researcher");
    bench_self_refinement(c, "Spec");

    bench_self_refinement(c, "Administration || Researcher || Machine");
}

fn refinement(c: &mut Criterion) {
    bench_refinement(c, "Researcher <= Spec // Administration // Machine");
    bench_refinement(c, "Machine <= Spec // Administration // Researcher");
    bench_refinement(c, "Administration <= Spec // Researcher // Machine");
    bench_refinement(c, "Administration || Researcher <= Spec // Machine");
    bench_refinement(c, "Researcher || Machine <= Spec // Administration");
    bench_refinement(c, "Machine || Administration <= Spec // Researcher");
}

fn not_refinement(c: &mut Criterion) {
    bench_non_refinement(c, "Adm2 <= Spec // Researcher // Machine");
    bench_non_refinement(c, "Machine <= Spec // Adm2 // Researcher");
    bench_non_refinement(c, "Adm2 || Researcher <= Spec // Machine");
}

fn send_query_same_components(c: &mut Criterion) {
    c.bench_function("send_query_same_components", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let backend = ConcreteEcdarBackend::default();
            let mut responses = vec![];
            for _ in 0..64 {
                let request = create_query_request("determinism: Machine");
                responses.push(backend.send_query(request));
            }

            for response in responses {
                _ = black_box(response.await);
            }
        });
    });
}

fn create_query_request(query: &str) -> Request<QueryRequest> {
    let json = std::fs::read_to_string(format!("{}/Components/Machine.json", PATH)).unwrap();

    Request::new(QueryRequest {
        user_id: 0,
        query_id: 0,
        query: String::from(query),
        components_info: Some(ComponentsInfo {
            components: vec![Component {
                rep: Some(Rep::Json(json)),
            }],
            components_hash: 0,
        }),
        ignored_input_outputs: None,
    })
}

criterion_group!(
    benches,
    self_refinement,
    refinement,
    not_refinement,
    send_query_same_components
);

criterion_main!(benches);
