use crate::debug_print;
use crate::DataReader::component_loader::ComponentContainer;
use crate::DataReader::component_loader::ComponentLoader;
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::xml_parser::parse_xml_from_str;
use crate::ModelObjects::component::Component;
use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::{Component as ProtobufComponent, ComponentsUpdateRequest};
use crate::ProtobufServer::ConcreteEcdarBackend;
use crate::ProtobufServer::ToGrpcResult;
use crate::System::input_enabler;
use anyhow::Result;
use std::cell::RefCell;
use tonic::{Request, Response};

impl ConcreteEcdarBackend {
    pub async fn handle_update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let update = request.into_inner();

        let component_container = self.get_components_lock()?;
        for proto_component in &update.components {
            let components = self.parse_components_if_some(proto_component)?;

            save_components(&component_container, components).as_grpc_result()?;
        }

        Ok(Response::new(()))
    }

    fn parse_components_if_some(
        &self,
        proto_component: &ProtobufComponent,
    ) -> Result<Vec<Component>, tonic::Status> {
        if let Some(rep) = &proto_component.rep {
            match rep {
                Rep::Json(json) => parse_json_component(json),
                Rep::Xml(xml) => parse_xml_components(xml),
            }
        } else {
            Ok(vec![])
        }
    }
}

fn parse_json_component(json: &str) -> Result<Vec<Component>, tonic::Status> {
    let comp = json_to_component(json).as_grpc_result()?;
    Ok(vec![comp])
}

fn parse_xml_components(xml: &str) -> Result<Vec<Component>, tonic::Status> {
    let (comps, _, _) = parse_xml_from_str(xml).as_grpc_result()?;
    Ok(comps)
}

fn save_components(
    component_container: &RefCell<ComponentContainer>,
    components: Vec<Component>,
) -> Result<()> {
    for mut component in components {
        debug_print!("Adding comp {} to container", component.get_name());
        component.create_edge_io_split();
        let inputs: Vec<_> = component
            .get_input_actions()?
            .into_iter()
            .map(|channel| channel.name)
            .collect();
        input_enabler::make_input_enabled(&mut component, &inputs).as_grpc_result()?;
        component_container.borrow_mut().save_component(component);
    }
    Ok(())
}
