use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::ComponentsUpdateRequest;
use tonic::{Request, Response};

use crate::DataReader::component_loader::ComponentLoader;
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::xml_parser::parse_xml_from_str;

use crate::ProtobufServer::ConcreteEcdarBackend;

impl ConcreteEcdarBackend {
    pub async fn handle_update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let update = request.into_inner();

        println!("Component count: {}", update.components.len());
        for comp in &update.components {
            if let Some(rep) = &comp.rep {
                match rep {
                    Rep::Json(json) => {
                        println!("json: {}", json);
                        let comp = json_to_component(&json);
                        let optimized_comp = comp.create_edge_io_split();

                        println!("Adding comp {} to container", optimized_comp.get_name());
                        {
                            let components = self.get_components_lock()?;
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                    }
                    Rep::Xml(xml) => {
                        let components = self.get_components_lock()?;
                        let (comps, _, _) = parse_xml_from_str(xml);

                        for component in comps {
                            println!("Adding comp {} to container", component.get_name());

                            let optimized_comp = component.create_edge_io_split();
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                    }
                }
            }
        }

        Ok(Response::new(()))
    }
}
