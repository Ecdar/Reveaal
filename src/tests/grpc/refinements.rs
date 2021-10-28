#[cfg(test)]
mod refinements {
    use crate::server::services::component::Rep;
    use crate::server::services::ecdar_backend_server::EcdarBackend;
    use crate::server::services::query_response;
    use crate::server::services::query_response::RefinementResult;
    use crate::server::services::{Component, ComponentsUpdateRequest, Query, QueryResponse};
    use crate::server::ConcreteEcdarBackend;
    use crate::tests::save_component::save_comp_helper::save_comp_helper::json_reconstructed_component_refines_base_self;
    use tonic::{Request, Response, Status};

    static CONJUN: &str = "samples/xml/conjun.xml";
    static ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[tokio::test]
    async fn send_query_fails_with_multiple_queries() {
        let backend = ConcreteEcdarBackend::default();
        let request = Request::new(Query {
            id: 0,
            query: String::from("refinement: A <= A; refinement: B <= B"),
            ignored_input_outputs: None,
        });

        let response = backend.send_query(request).await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn send_query_fails_with_no_queries() {
        let backend = ConcreteEcdarBackend::default();
        let request = Request::new(Query {
            id: 0,
            query: String::from(""),
            ignored_input_outputs: None,
        });

        let response = backend.send_query(request).await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn send_empty_update_components() {
        let backend = ConcreteEcdarBackend::default();
        let request = Request::new(ComponentsUpdateRequest {
            components: vec![],
            etag: 0,
        });

        let response = backend.update_components(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn send_component() {
        let backend = ConcreteEcdarBackend::default();
        let json =
            std::fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();
        let request = Request::new(ComponentsUpdateRequest {
            components: vec![Component {
                rep: Some(Rep::Json(json)),
            }],
            etag: 0,
        });

        let response = backend.update_components(request).await;
        assert!(response.is_ok());

        let components = backend.components.lock().unwrap();
        let component_count = components.borrow_mut().loaded_components.len();

        assert_eq!(component_count, 1);
    }

    #[tokio::test]
    async fn send_two_components() {
        let backend = ConcreteEcdarBackend::default();
        let json1 =
            std::fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();
        let json2 = std::fs::read_to_string(format!("{}/Components/Adm2.json", ECDAR_UNI)).unwrap();
        let request = Request::new(ComponentsUpdateRequest {
            components: vec![
                Component {
                    rep: Some(Rep::Json(json1)),
                },
                Component {
                    rep: Some(Rep::Json(json2)),
                },
            ],
            etag: 0,
        });

        let response = backend.update_components(request).await;
        assert!(response.is_ok());

        let components = backend.components.lock().unwrap();
        let component_count = components.borrow_mut().loaded_components.len();

        assert_eq!(component_count, 2);
    }

    #[tokio::test]
    async fn send_xml_components() {
        let backend = ConcreteEcdarBackend::default();
        let xml = std::fs::read_to_string(CONJUN).unwrap();
        let request = Request::new(ComponentsUpdateRequest {
            components: vec![Component {
                rep: Some(Rep::Xml(xml)),
            }],
            etag: 0,
        });

        let response = backend.update_components(request).await;
        assert!(response.is_ok());

        let components = backend.components.lock().unwrap();
        let component_count = components.borrow_mut().loaded_components.len();

        assert_eq!(component_count, 14);
    }

    #[tokio::test]
    async fn send_self_refinement_query() {
        let backend = ConcreteEcdarBackend::default();
        let json =
            std::fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();
        let update_request = Request::new(ComponentsUpdateRequest {
            components: vec![Component {
                rep: Some(Rep::Json(json)),
            }],
            etag: 0,
        });

        let update_response = backend.update_components(update_request).await;
        assert!(update_response.is_ok());

        let query = String::from("refinement: Machine <= Machine");
        let query_request = Request::new(Query {
            id: 0,
            query,
            ignored_input_outputs: None,
        });

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_ok());

        let query_result = query_response.unwrap().into_inner();

        if let Some(result) = query_result.result {
            match result {
                query_response::Result::Refinement(refine) => assert!(refine.success),
                _ => assert!(false),
            }
        } else {
            assert!(false);
        }
    }
}
