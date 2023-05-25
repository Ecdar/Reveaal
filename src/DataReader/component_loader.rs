use log::warn;
use lru::LruCache;

use crate::component::Automaton;
use crate::xml_parser;
use crate::DataReader::json_reader;
use crate::DataReader::json_writer::automaton_to_json_file;
use crate::DataReader::xml_parser::parse_xml_from_file;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::ProtobufServer::services;
use crate::ProtobufServer::services::query_request::Settings;
use crate::System::input_enabler;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use super::proto_reader::components_info_to_automata;

type AutomataMap = HashMap<String, Automaton>;

struct AutomataTuple {
    automata_hash: u32,
    automata_map: Arc<AutomataMap>,
}

/// A struct used for caching the models.
#[derive(Debug, Clone)]
pub struct ModelCache {
    // TODO: A concurrent lru may be faster to use and cause less prone to lock contention.
    cache: Arc<Mutex<LruCache<i32, AutomataTuple>>>,
}

impl Default for ModelCache {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::<i32, AutomataTuple>::new(
                NonZeroUsize::new(100).unwrap(),
            ))),
        }
    }
}

impl ModelCache {
    /// A Method that creates a new cache with a given size limit.
    ///
    /// # Arguments
    ///
    /// * `cache_size` - A number representing the number of users that can be cached simultaneusly.
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::<i32, AutomataTuple>::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
        }
    }

    /// A Method that returns the model from the cache.
    ///
    /// # Arguments
    ///
    /// * `automata_hash` - A hash of the automata
    pub fn get_model(&self, user_id: i32, automata_hash: u32) -> Option<AutomataContainer> {
        if automata_hash == 0 {
            warn!("The automaton has no hash (0), so we assume it should not be cached.");
            return None;
        }

        let mut cache = self.cache.lock().unwrap();

        let automata = cache.get(&user_id);

        automata.and_then(|automata_pair| {
            if automata_pair.automata_hash == automata_hash {
                Some(AutomataContainer::new(Arc::clone(
                    &automata_pair.automata_map,
                )))
            } else {
                None
            }
        })
    }

    /// A method that inserts a new model into the cache.
    ///
    /// # Arguments
    ///
    /// * `automata_hash` - A hash of the automata
    /// * `container_automata` - The `AutomataContainer's` loaded automata (aka Model) to be cached.
    pub fn insert_model(
        &mut self,
        user_id: i32,
        automata_hash: u32,
        container_automata: Arc<AutomataMap>,
    ) -> AutomataContainer {
        if automata_hash == 0 {
            warn!("The automaton has no hash (0), so we assume it should not be cached.");
            return AutomataContainer::new(container_automata);
        }

        self.cache.lock().unwrap().put(
            user_id,
            AutomataTuple {
                automata_hash,
                automata_map: Arc::clone(&container_automata),
            },
        );

        AutomataContainer::new(container_automata)
    }
}

pub trait AutomataLoader {
    fn get_automaton(&mut self, automaton_name: &str) -> &Automaton;
    fn save_automaton(&mut self, automaton: Automaton);
    fn get_settings(&self) -> &Settings;
}

#[derive(Debug, Default, Clone)]
pub struct AutomataContainer {
    pub loaded_automata: Arc<AutomataMap>,
    settings: Option<Settings>,
}

impl AutomataLoader for AutomataContainer {
    fn get_automaton(&mut self, automaton_name: &str) -> &Automaton {
        if let Some(automaton) = self.loaded_automata.get(automaton_name) {
            assert_eq!(automaton_name, automaton.get_name());
            automaton
        } else {
            panic!("The automaton '{}' could not be retrieved", automaton_name);
        }
    }
    fn save_automaton(&mut self, _automaton: Automaton) {
        //Intentionally left blank (no-op func)
    }

    fn get_settings(&self) -> &Settings {
        self.settings.as_ref().unwrap()
    }
}

impl AutomataContainer {
    pub fn new(map: Arc<AutomataMap>) -> Self {
        AutomataContainer {
            loaded_automata: map,
            settings: None,
        }
    }

    /// Creates a [`AutomataContainer`] from a [`services::ComponentsInfo`].
    pub fn from_info(
        components_info: &services::ComponentsInfo,
    ) -> Result<AutomataContainer, tonic::Status> {
        let automata = components_info_to_automata(components_info);
        let automata_container = Self::from_automata(automata);
        Ok(automata_container)
    }

    /// Creates a [`AutomataContainer`] from a [`Vec`] of [`Automaton`]s
    pub fn from_automata(automata: Vec<Automaton>) -> AutomataContainer {
        let mut comp_hashmap = HashMap::<String, Automaton>::new();
        for mut automaton in automata {
            log::trace!("Adding comp {} to container", automaton.get_name());
            let inputs: Vec<_> = automaton.get_input_actions();
            input_enabler::make_input_enabled(&mut automaton, &inputs);
            comp_hashmap.insert(automaton.get_name().to_string(), automaton);
        }
        AutomataContainer::new(Arc::new(comp_hashmap))
    }

    /// Sets the settings
    pub(crate) fn set_settings(&mut self, settings: Settings) {
        self.settings = Some(settings);
    }
}

pub fn parse_automata_if_some(
    proto_component: &services::Component,
) -> Result<Vec<Automaton>, tonic::Status> {
    if let Some(rep) = &proto_component.rep {
        match rep {
            services::component::Rep::Json(json) => parse_json_automaton(json),
            services::component::Rep::Xml(xml) => Ok(parse_xml_automata(xml)),
        }
    } else {
        Ok(vec![])
    }
}

fn parse_json_automaton(json: &str) -> Result<Vec<Automaton>, tonic::Status> {
    match json_reader::json_to_automaton(json) {
        Ok(comp) => Ok(vec![comp]),
        Err(_) => Err(tonic::Status::invalid_argument(
            "Failed to parse json automaton",
        )),
    }
}

fn parse_xml_automata(xml: &str) -> Vec<Automaton> {
    let (comps, _, _) = xml_parser::parse_xml_from_str(xml);
    comps
}

pub trait ProjectLoader: AutomataLoader {
    fn get_declarations(&self) -> &SystemDeclarations;
    fn get_queries(&self) -> &Vec<Query>;
    fn get_project_path(&self) -> &str;
    fn to_comp_loader(self: Box<Self>) -> Box<dyn AutomataLoader>;
}

pub struct JsonProjectLoader {
    project_path: String,
    loaded_automata: AutomataMap,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
    settings: Settings,
}

impl AutomataLoader for JsonProjectLoader {
    fn get_automaton(&mut self, automaton_name: &str) -> &Automaton {
        if !self.is_automaton_loaded(automaton_name) {
            self.load_automaton(automaton_name);
        }

        if let Some(automaton) = self.loaded_automata.get(automaton_name) {
            assert_eq!(automaton_name, automaton.get_name());
            automaton
        } else {
            panic!("The automaton '{}' could not be retrieved", automaton_name);
        }
    }

    fn save_automaton(&mut self, automaton: Automaton) {
        automaton_to_json_file(&self.project_path, &automaton);
        self.loaded_automata
            .insert(automaton.get_name().clone(), automaton);
    }

    fn get_settings(&self) -> &Settings {
        &self.settings
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

    fn to_comp_loader(self: Box<Self>) -> Box<dyn AutomataLoader> {
        self
    }
}

impl JsonProjectLoader {
    pub fn new_loader(project_path: String, settings: Settings) -> Box<dyn ProjectLoader> {
        let system_declarations = json_reader::read_system_declarations(&project_path).unwrap();
        let queries = json_reader::read_queries(&project_path).unwrap();

        Box::new(JsonProjectLoader {
            project_path,
            loaded_automata: HashMap::new(),
            system_declarations,
            queries,
            settings,
        })
    }

    fn load_automaton(&mut self, automaton_name: &str) {
        let mut automaton = json_reader::read_json_automaton(&self.project_path, automaton_name);

        let opt_inputs = self
            .get_declarations()
            .get_automaton_inputs(automaton.get_name());
        if let Some(inputs) = opt_inputs {
            input_enabler::make_input_enabled(&mut automaton, inputs);
        }

        self.loaded_automata
            .insert(String::from(automaton_name), automaton);
    }

    fn is_automaton_loaded(&self, automaton_name: &str) -> bool {
        self.loaded_automata.contains_key(automaton_name)
    }
}

pub struct XmlProjectLoader {
    project_path: String,
    loaded_automata: AutomataMap,
    system_declarations: SystemDeclarations,
    queries: Vec<Query>,
    settings: Settings,
}

impl AutomataLoader for XmlProjectLoader {
    fn get_automaton(&mut self, automaton_name: &str) -> &Automaton {
        if let Some(automaton) = self.loaded_automata.get(automaton_name) {
            assert_eq!(automaton_name, automaton.get_name());
            automaton
        } else {
            panic!("The automaton '{}' could not be retrieved", automaton_name);
        }
    }

    fn save_automaton(&mut self, _: Automaton) {
        panic!("Saving automata is not supported for XML projects")
    }

    fn get_settings(&self) -> &Settings {
        &self.settings
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

    fn to_comp_loader(self: Box<Self>) -> Box<dyn AutomataLoader> {
        self
    }
}

impl XmlProjectLoader {
    pub fn new_loader(project_path: String, settings: Settings) -> Box<dyn ProjectLoader> {
        let (automata, system_declarations, queries) = parse_xml_from_file(&project_path);

        let mut map = HashMap::<String, Automaton>::new();
        for mut automaton in automata {
            let opt_inputs = system_declarations.get_automaton_inputs(automaton.get_name());
            if let Some(opt_inputs) = opt_inputs {
                input_enabler::make_input_enabled(&mut automaton, opt_inputs);
            }

            let name = String::from(automaton.get_name());
            map.insert(name, automaton);
        }

        Box::new(XmlProjectLoader {
            project_path,
            loaded_automata: map,
            system_declarations,
            queries,
            settings,
        })
    }
}
