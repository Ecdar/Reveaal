use crate::DataReader::component_loader::ComponentContainer;
use crate::DataReader::component_loader::ComponentLoader;
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::xml_parser::parse_xml_from_str;
use crate::ModelObjects::component::Component;
use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::{Component as ProtobufComponent, ComponentsUpdateRequest};
use crate::ProtobufServer::ConcreteEcdarBackend;
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
            let components = self.parse_components_if_some(proto_component);

            save_components(&component_container, components);
        }

        Ok(Response::new(()))
    }

    fn parse_components_if_some(&self, proto_component: &ProtobufComponent) -> Vec<Component> {
        if let Some(rep) = &proto_component.rep {
            self.parse_components(rep)
        } else {
            vec![]
        }
    }

    fn parse_components(&self, component_representation: &Rep) -> Vec<Component> {
        match component_representation {
            Rep::Json(json) => {
                let comp = json_to_component(&json);

                vec![comp]
            }
            Rep::Xml(xml) => {
                let (comps, _, _) = parse_xml_from_str(&xml);

                comps
            }
        }
    }
}

fn save_components(component_container: &RefCell<ComponentContainer>, components: Vec<Component>) {
    for component in components {
        println!("Adding comp {} to container", component.get_name());
        let optimized_comp = component.create_edge_io_split();
        component_container
            .borrow_mut()
            .save_component(optimized_comp);
    }
}
