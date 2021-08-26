use crate::component::Component;
use crate::DataReader::json_reader;
use crate::DataReader::xml_parser::parse_xml;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::input_enabler;
use std::collections::HashMap;

pub trait ProjectLoader {
    fn get_component(&mut self, component_name: &str) -> &Component;
    fn unload_component(&mut self, component_name: &str);
    fn get_declarations(&self) -> &SystemDeclarations;
    fn get_queries(&self) -> &Vec<Query>;
    fn get_project_path(&self) -> &str;
}

pub struct JsonProjectLoader {
    project_path: String,
    loaded_components: HashMap<String, Component>,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
}

impl ProjectLoader for JsonProjectLoader {
    fn get_component(&mut self, component_name: &str) -> &Component {
        if !self.is_component_loaded(component_name) {
            self.load_component(component_name);
        }

        if let Some(component) = self.loaded_components.get(component_name) {
            &component
        } else {
            panic!("The component '{}' could not be retrieved", component_name);
        }
    }

    fn unload_component(&mut self, component_name: &str) {
        self.loaded_components.remove(component_name);
    }

    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &str {
        &self.project_path
    }
}

impl JsonProjectLoader {
    pub fn new(project_path: String) -> Box<dyn ProjectLoader> {
        let system_declarations = json_reader::read_system_declarations(&project_path).unwrap();
        let queries = json_reader::read_queries(&project_path).unwrap();

        Box::new(JsonProjectLoader {
            project_path,
            loaded_components: HashMap::new(),
            system_declarations,
            queries,
        })
    }

    fn load_component(&mut self, component_name: &str) {
        let component = json_reader::read_json_component(&self.project_path, component_name);

        let mut optimized_comp = component.create_edge_io_split();
        input_enabler::make_input_enabled(&mut optimized_comp, self.get_declarations());

        self.loaded_components
            .insert(String::from(component_name), optimized_comp);
    }

    fn is_component_loaded(&self, component_name: &str) -> bool {
        self.loaded_components.contains_key(component_name)
    }
}

pub struct XmlProjectLoader {
    project_path: String,
    loaded_components: HashMap<String, Component>,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
}

impl ProjectLoader for XmlProjectLoader {
    fn get_component(&mut self, component_name: &str) -> &Component {
        if let Some(component) = self.loaded_components.get(component_name) {
            &component
        } else {
            panic!("The component '{}' could not be retrieved", component_name);
        }
    }

    fn unload_component(&mut self, _: &str) {
        panic!("unloading and loading individual components isnt permitted in XML")
    }

    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &str {
        &self.project_path
    }
}

impl XmlProjectLoader {
    pub fn new(project_path: String) -> Box<dyn ProjectLoader> {
        let (comps, system_declarations, queries) = parse_xml(&project_path);

        let mut map = HashMap::<String, Component>::new();
        for component in comps {
            let mut optimized_comp = component.create_edge_io_split();
            input_enabler::make_input_enabled(&mut optimized_comp, &system_declarations);

            let name = String::from(optimized_comp.get_name());
            map.insert(name, optimized_comp);
        }

        Box::new(XmlProjectLoader {
            project_path,
            loaded_components: map,
            system_declarations,
            queries,
        })
    }
}
