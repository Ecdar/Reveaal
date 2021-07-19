use crate::ModelObjects::component::{Channel, Component};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct SystemDeclarations {
    pub(crate) name: String,
    #[serde(deserialize_with = "decode_sync_type")]
    pub(crate) declarations: SystemSpecification,
}
#[allow(dead_code)]
impl SystemDeclarations {
    pub fn get_declarations(&self) -> &SystemSpecification {
        &self.declarations
    }
    pub fn get_mut_declarations(&mut self) -> &mut SystemSpecification {
        &mut self.declarations
    }

    pub fn add_component(&mut self, comp: &Component) {
        self.declarations.input_actions.insert(
            comp.get_name().clone(),
            comp.get_input_actions()
                .into_iter()
                .map(|channel| channel.name)
                .collect(),
        );
        self.declarations.output_actions.insert(
            comp.get_name().clone(),
            comp.get_output_actions()
                .into_iter()
                .map(|channel| channel.name)
                .collect(),
        );
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemSpecification {
    pub(crate) components: Vec<String>,
    pub(crate) input_actions: HashMap<String, Vec<String>>,
    pub(crate) output_actions: HashMap<String, Vec<String>>,
}
#[allow(dead_code)]
impl SystemSpecification {
    pub fn get_components(&self) -> &Vec<String> {
        &self.components
    }
    pub fn get_mut_components(&mut self) -> &mut Vec<String> {
        &mut self.components
    }
    pub fn get_input_actions(&self) -> &HashMap<String, Vec<String>> {
        &self.input_actions
    }
    pub fn get_mut_input_actions(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.input_actions
    }
    pub fn get_output_actions(&self) -> &HashMap<String, Vec<String>> {
        &self.output_actions
    }
    pub fn get_mut_output_actions(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.output_actions
    }
}

/// Function used for deserializing system declarations
fn decode_sync_type<'de, D>(deserializer: D) -> Result<SystemSpecification, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut first_run = true;
    let decls: Vec<String> = s.split('\n').map(|s| s.into()).collect();
    let mut input_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut output_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut components: Vec<String> = vec![];

    let mut component_names: Vec<String> = vec![];

    for declaration in &decls {
        //skip comments
        if declaration.starts_with("//") || declaration.is_empty() {
            continue;
        }

        if !declaration.is_empty() {
            if first_run {
                let component_decls = &declaration;

                component_names = component_decls.split(' ').map(|s| s.into()).collect();

                if component_names[0] == "system" {
                    //do not include element 0 as that is the system keyword
                    for name in component_names.iter_mut().skip(1) {
                        let s = name.replace(",", "");
                        let s_cleaned = s.replace(";", "");
                        *name = s_cleaned.clone();
                        components.push(s_cleaned);
                    }
                    first_run = false;
                } else {
                    panic!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
                }
            }

            let split_string: Vec<String> = declaration.split(' ').map(|s| s.into()).collect();
            if split_string[0].as_str() == "IO" {
                let component_name = split_string[1].clone();

                if component_names.contains(&component_name) {
                    for split_str in split_string.iter().skip(2) {
                        let s = split_str.replace("{", "");
                        let p = s.replace("}", "");
                        let comp_actions: Vec<String> = p.split(',').map(|s| s.into()).collect();
                        for action in comp_actions {
                            if action.is_empty() {
                                continue;
                            }
                            if action.ends_with('?') {
                                let r = action.replace("?", "");
                                if let Some(Channel_vec) = input_actions.get_mut(&component_name) {
                                    Channel_vec.push(r)
                                } else {
                                    let Channel_vec = vec![r];
                                    input_actions.insert(component_name.clone(), Channel_vec);
                                }
                            } else if action.ends_with('!') {
                                let r = action.replace("!", "");
                                if let Some(Channel_vec) = output_actions.get_mut(&component_name) {
                                    Channel_vec.push(r.clone())
                                } else {
                                    let Channel_vec = vec![r.clone()];
                                    output_actions.insert(component_name.clone(), Channel_vec);
                                }
                            } else {
                                panic!("Channel type not defined for Channel {:?}", action)
                            }
                        }
                    }
                } else {
                    panic!("Was not able to find component name: {:?} in declared component names: {:?}", component_name, component_names)
                }
            }
        }
    }
    Ok(SystemSpecification {
        components,
        input_actions,
        output_actions,
    })
}
