use crate::component::Component;
use crate::DataReader::json_reader;
use crate::DataReader::json_writer::component_to_json_file;
use crate::DataReader::xml_parser::parse_xml_from_file;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::input_enabler;
use crate::{bail, context};
use anyhow::Result;
use std::collections::HashMap;

pub trait ComponentLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component>;
    fn save_component(&mut self, component: Component) -> Result<()>;
    fn unload_component(&mut self, component_name: &str);
}

#[derive(Debug, Default, Clone)]
pub struct ComponentContainer {
    pub loaded_components: HashMap<String, Component>,
}

impl ComponentLoader for ComponentContainer {
    fn get_component(&mut self, component_name: &str) -> Result<&Component> {
        context!(
            self.loaded_components.get(component_name),
            "The component '{}' could not be retrieved",
            component_name
        )
    }
    fn save_component(&mut self, component: Component) -> Result<()> {
        self.unload_component(&component.name);
        self.loaded_components
            .insert(component.get_name().clone(), component);
        Ok(())
    }
    fn unload_component(&mut self, component_name: &str) {
        self.loaded_components.remove(component_name);
    }
}

pub trait ProjectLoader: ComponentLoader {
    fn get_declarations(&self) -> &SystemDeclarations;
    fn get_queries(&self) -> &Vec<Query>;
    fn get_project_path(&self) -> &str;
    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader>;
}

pub struct JsonProjectLoader {
    project_path: String,
    loaded_components: HashMap<String, Component>,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
}

impl ComponentLoader for JsonProjectLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component> {
        if !self.is_component_loaded(component_name) {
            self.load_component(component_name)?;
        }

        context!(
            self.loaded_components.get(component_name),
            "The component '{}' could not be retrieved",
            component_name
        )
    }

    fn save_component(&mut self, component: Component) -> Result<()> {
        self.unload_component(&component.name);
        component_to_json_file(&self.project_path, &component)?;
        self.loaded_components
            .insert(component.get_name().clone(), component);
        Ok(())
    }

    fn unload_component(&mut self, component_name: &str) {
        self.loaded_components.remove(component_name);
    }
}

impl ProjectLoader for JsonProjectLoader {
    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &str {
        &self.project_path
    }

    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader> {
        self
    }
}

impl JsonProjectLoader {
    pub fn new(project_path: String) -> Result<Box<dyn ProjectLoader>> {
        let system_declarations = json_reader::read_system_declarations(&project_path)?;
        let queries = json_reader::read_queries(&project_path)?;

        Ok(Box::new(JsonProjectLoader {
            project_path,
            loaded_components: HashMap::new(),
            system_declarations,
            queries,
        }))
    }

    fn load_component(&mut self, component_name: &str) -> Result<()> {
        let mut component = json_reader::read_json_component(&self.project_path, component_name)?;

        component.create_edge_io_split();

        let opt_inputs = self
            .get_declarations()
            .get_component_inputs(component.get_name());
        if let Some(inputs) = opt_inputs {
            input_enabler::make_input_enabled(&mut component, &inputs)?;
        }

        self.loaded_components
            .insert(String::from(component_name), component);
        Ok(())
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

impl ComponentLoader for XmlProjectLoader {
    fn get_component(&mut self, component_name: &str) -> Result<&Component> {
        context!(
            self.loaded_components.get(component_name),
            "The component '{}' could not be retrieved",
            component_name
        )
    }

    fn save_component(&mut self, _: Component) -> Result<()> {
        bail!("Saving components is not supported for XML projects")
    }

    fn unload_component(&mut self, comp_name: &str) {
        self.loaded_components.remove(comp_name);
    }
}

impl ProjectLoader for XmlProjectLoader {
    fn get_declarations(&self) -> &SystemDeclarations {
        &self.system_declarations
    }

    fn get_queries(&self) -> &Vec<Query> {
        &self.queries
    }

    fn get_project_path(&self) -> &str {
        &self.project_path
    }

    fn to_comp_loader(self: Box<Self>) -> Box<dyn ComponentLoader> {
        self
    }
}

impl XmlProjectLoader {
    pub fn new(project_path: String) -> Result<Box<dyn ProjectLoader>> {
        let (comps, system_declarations, queries) = parse_xml_from_file(&project_path)?;

        let mut map = HashMap::<String, Component>::new();
        for mut component in comps {
            component.create_edge_io_split();

            let opt_inputs = system_declarations.get_component_inputs(component.get_name());
            if let Some(inputs) = opt_inputs {
                input_enabler::make_input_enabled(&mut component, inputs)?;
            }

            let name = String::from(component.get_name());
            map.insert(name, component);
        }

        Ok(Box::new(XmlProjectLoader {
            project_path,
            loaded_components: map,
            system_declarations,
            queries,
        }))
    }
}
