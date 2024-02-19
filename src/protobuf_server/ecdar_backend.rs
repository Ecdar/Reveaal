use crate::protobuf_server::services::ecdar_backend_server::EcdarBackend;

use crate::data_reader::component_loader::ModelCache;
use crate::protobuf_server::services::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use futures::executor::block_on;
use futures::FutureExt;
use std::panic::UnwindSafe;
use std::sync::atomic::{AtomicI32, Ordering};
use tonic::{Request, Response, Status};

use rayon::{ThreadPool, ThreadPoolBuilder};

#[derive(Debug)]
pub struct ConcreteEcdarBackend {
    thread_pool: ThreadPool,
    model_cache: ModelCache,
    num: AtomicI32,
}

impl ConcreteEcdarBackend {
    pub fn new(thread_count: usize, cache_size: usize) -> Self {
        ConcreteEcdarBackend {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .unwrap(),
            model_cache: ModelCache::new(cache_size),
            num: AtomicI32::new(1),
        }
    }
}

impl Default for ConcreteEcdarBackend {
    fn default() -> Self {
        ConcreteEcdarBackend {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .build()
                .unwrap(),

            model_cache: ModelCache::default(),
            num: AtomicI32::new(1),
        }
    }
}

async fn catch_unwind<T, O>(future: T) -> Result<Response<O>, Status>
where
    T: UnwindSafe + futures::Future<Output = Result<O, Status>>,
{
    fn downcast_to_string(e: Box<dyn std::any::Any + Send>) -> String {
        match e.downcast::<String>() {
            Ok(v) => *v,
            Err(e) => match e.downcast::<&str>() {
                Ok(v) => v.to_string(),
                _ => "Unknown Source of Error".to_owned(),
            },
        }
    }

    future
        .catch_unwind()
        .await
        .unwrap_or_else(|e| {
            Err(Status::internal(format!(
                "{}, please report this bug to the developers",
                downcast_to_string(e)
            )))
        })
        .map(Response::new)
}

impl ConcreteEcdarBackend {}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn get_user_token(
        &self,
        _request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        let id = self.num.fetch_add(1, Ordering::SeqCst);
        let token_response = UserTokenResponse { user_id: id };
        Ok(Response::new(token_response))
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        async fn async_query(
            request: QueryRequest,
            cache: ModelCache,
        ) -> Result<QueryResponse, Status> {
            ConcreteEcdarBackend::handle_send_query(request, cache)
        }
        let cache = self.model_cache.clone();

        self.thread_pool
            .install(|| block_on(catch_unwind(async_query(request.into_inner(), cache))))

        // TODO: Test whether there is a large performance difference between block_on and the non-catching commented out code below
        // self.thread_pool.install(|| {
        //     ConcreteEcdarBackend::handle_send_query(request.into_inner(), cache).map(Response::new)
        // })
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        async fn async_start_simulation(
            request: SimulationStartRequest,
            cache: ModelCache,
        ) -> Result<SimulationStepResponse, Status> {
            ConcreteEcdarBackend::handle_start_simulation(request, cache)
        }

        catch_unwind(async_start_simulation(
            request.into_inner(),
            self.model_cache.clone(),
        ))
        .await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        async fn async_simulation_step(
            request: SimulationStepRequest,
            cache: ModelCache,
        ) -> Result<SimulationStepResponse, Status> {
            ConcreteEcdarBackend::handle_take_simulation_step(request, cache)
        }

        catch_unwind(async_simulation_step(
            request.into_inner(),
            self.model_cache.clone(),
        ))
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::protobuf_server::services::component::Rep;
    use crate::protobuf_server::services::{
        query_response, Component, ComponentsInfo, QueryRequest, SimulationStartRequest,
        SimulationStepRequest, SimulationStepResponse,
    };
    use crate::protobuf_server::ConcreteEcdarBackend;
    use crate::protobuf_server::{self, services::ecdar_backend_server::EcdarBackend};
    use std::fs;
    use test_case::test_case;
    use tonic::{Request, Response, Status};

    //const CONJUN: &str = "samples/xml/conjun.xml";
    const ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    impl SimulationStartRequest {
        pub fn new(component_names: &[&str], components_path: &str, composition: &str) -> Self {
            let simulation_info = protobuf_server::services::SimulationInfo::new(
                component_names,
                components_path,
                composition,
            );
            SimulationStartRequest {
                simulation_info: Some(simulation_info),
            }
        }
    }

    pub fn construct_step_requests(
        component_names: &[&str],
        components_path: &str,
        composition: &str,
        last_response: Result<Response<SimulationStepResponse>, Status>,
    ) -> impl Iterator<Item = SimulationStepRequest> {
        let simulation_info = protobuf_server::services::SimulationInfo::new(
            component_names,
            components_path,
            composition,
        );
        let last_response = last_response.unwrap().into_inner();
        last_response
            .new_decision_points
            .into_iter()
            .map(move |d| SimulationStepRequest {
                simulation_info: Some(simulation_info.clone()),
                chosen_decision: Some(d),
            })
    }

    impl protobuf_server::services::SimulationInfo {
        fn new(component_names: &[&str], components_path: &str, composition: &str) -> Self {
            let json_components: Vec<_> = component_names
                .iter()
                .map(|component_name| Component {
                    rep: Some(Rep::Json(
                        fs::read_to_string(format!(
                            "{}/Components/{}.json",
                            components_path, component_name
                        ))
                        .unwrap(),
                    )),
                })
                .collect();

            protobuf_server::services::SimulationInfo {
                user_id: 0,
                component_composition: composition.to_string(),
                components_info: Some(ComponentsInfo {
                    components: json_components,
                    components_hash: 0,
                }),
            }
        }
    }

    fn construct_query_request_for_cache(
        query: &str,
    ) -> (Request<QueryRequest>, Request<QueryRequest>) {
        let json = fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();

        let normal_request = Request::new(QueryRequest {
            user_id: 0,
            query_id: 0,
            query: String::from(query),
            components_info: Some(ComponentsInfo {
                components: vec![Component {
                    rep: Some(Rep::Json(json)),
                }],
                components_hash: 1,
            }),
            settings: Some(crate::DEFAULT_SETTINGS),
        });

        //TODO There is some fancy rust syntax to make a clone of the above with minor alterations.
        let empty_component_request = Request::new(QueryRequest {
            user_id: 0,
            query_id: 0,
            query: String::from(query),
            components_info: Some(ComponentsInfo {
                components: vec![],
                components_hash: 1,
            }),
            settings: Some(crate::DEFAULT_SETTINGS),
        });
        (normal_request, empty_component_request)
    }

    fn construct_query_request(query: &str) -> Request<QueryRequest> {
        let json = fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();

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
            settings: Some(crate::DEFAULT_SETTINGS),
        })
    }

    #[test_case(
    &["Machine"],
    "samples/json/EcdarUniversity",
    "(Machine)"
    )]
    #[test_case(
    &["HalfAdm1", "HalfAdm2"],
    "samples/json/EcdarUniversity",
    "(HalfAdm1 && HalfAdm2)"
    )]
    #[test_case(
    &["Administration", "Machine", "Researcher"],
    "samples/json/EcdarUniversity",
    "(Administration || Machine || Researcher)"
    )]
    #[test_case(
    &["Spec", "Machine"],
    "samples/json/EcdarUniversity",
    "(Spec // Machine)"
    )]
    #[test_case(
    &["Machine", "Spec", "Researcher"],
    "samples/json/EcdarUniversity",
    "(Spec // Machine // Researcher)"
    )]
    #[test_case(
    &["HalfAdm1", "HalfAdm2", "Machine", "Researcher"],
    "samples/json/EcdarUniversity",
    "((HalfAdm1 && HalfAdm2) || Machine || Researcher)"
    )]
    #[tokio::test]
    async fn start_simulation_then_take_simulation_step(
        component_names: &[&str],
        components_path: &str,
        composition: &str,
    ) {
        // Arrange
        let backend = ConcreteEcdarBackend::default();
        let request = Request::new(SimulationStartRequest::new(
            component_names,
            components_path,
            composition,
        ));

        // Act
        let response = backend.start_simulation(request).await;

        // Arrange
        for request in
            construct_step_requests(component_names, components_path, composition, response)
        {
            let request = Request::new(request);

            // Act
            let response = backend.take_simulation_step(request).await;

            // Assert
            assert!(response.is_ok(), "Response was not ok: {:?}", response);

            for request2 in
                construct_step_requests(component_names, components_path, composition, response)
            {
                let request2 = Request::new(request2);

                // Act
                let response2 = backend.take_simulation_step(request2).await;

                // Assert
                assert!(response2.is_ok(), "Response was not ok: {:?}", response2);
            }
        }
    }

    #[tokio::test]
    async fn send_self_refinement_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("refinement: Machine <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_ok());

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_consistency_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("consistency: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_syntax_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("syntax: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_determinism_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("determinism: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_query_using_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        // Normal query request, including component.
        let query_response = backend.send_query(query_request.0).await;

        let query_result = query_response.unwrap().into_inner();
        let normal_result = query_result.result.unwrap();

        // Component should be cached now.
        // Query without component utilizing cache
        let query_response = backend.send_query(query_request.1).await;

        let query_result = query_response.unwrap().into_inner();
        let cache_result = query_result.result.unwrap();

        // Compare normal and cache response.
        assert_eq!(normal_result, cache_result);
    }

    #[tokio::test]
    async fn send_query_not_in_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        // Cache request
        let query_response = backend.send_query(query_request.1).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();

        match result {
            query_response::Result::ComponentsNotInCache(_) => {}
            _ => panic!("Expected failure, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_query_different_users_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        let _ = backend.send_query(query_request.0).await;

        let user_1_request = Request::new(QueryRequest {
            user_id: 1,
            ..query_request.1.into_inner()
        });

        let query_response = backend.send_query(user_1_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();

        match result {
            query_response::Result::ComponentsNotInCache(_) => {}
            _ => panic!("Expected failure, got {:?}", result),
        }
    }
    /// Ensure that the backend does not crash when a query panics
    #[tokio::test]
    async fn send_panic_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("refinement: Machine | <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_err());
    }

    /// Ensure that the backend can recover from panics entirely
    #[tokio::test]
    async fn send_after_panic_query() {
        let backend = ConcreteEcdarBackend::default();
        for _ in 0..5 {
            let query_request = construct_query_request("refinement: Machine | <= Machine");
            let query_response = backend.send_query(query_request).await;
            assert!(query_response.is_err());
        }
        let query_request = construct_query_request("refinement: Machine <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_ok());

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }
}
