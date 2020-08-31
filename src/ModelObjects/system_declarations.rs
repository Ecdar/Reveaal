use std::collections::HashMap;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone)]
pub struct SystemDeclarations {
    name : String,
    #[serde(deserialize_with = "decode_sync_type")]
    declarations : SystemSpecification,
}

impl SystemDeclarations {
    pub fn get_declarations(&self) -> &SystemSpecification {
        &self.declarations
    }
    pub fn get_mut_declarations(&mut self) -> &mut SystemSpecification {
        &mut self.declarations
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemSpecification {
    components : Vec<String>,
    input_actions : HashMap<String, Vec<String>>,
    output_actions : HashMap<String, Vec<String>>,
}

impl SystemSpecification {
    pub fn get_components(&self) -> &Vec<String> {
        &self.components
    }
    pub fn get_input_actions(&self) -> &HashMap<String, Vec<String>>{
        &self.input_actions
    }
    pub fn get_mut_input_actions(&mut self) -> &mut HashMap<String, Vec<String>>{
        &mut self.input_actions
    }
    pub fn get_output_actions(&self) -> &HashMap<String, Vec<String>>{
        &self.output_actions
    }
    pub fn get_mut_output_actions(&mut self) -> &mut HashMap<String, Vec<String>>{
        &mut self.output_actions
    }
}


//Function used for deserializing system declarations
fn decode_sync_type<'de, D>(deserializer: D) -> Result<SystemSpecification, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut first_run = true;
    let decls: Vec<String> = s.split("\n").map(|s| s.into()).collect();
    let mut input_actions : HashMap<String, Vec<String>> = HashMap::new();
    let mut output_actions : HashMap<String, Vec<String>> = HashMap::new();
    let mut components: Vec<String> = vec![];

    let mut component_names: Vec<String> = vec![];


    for i  in 0..decls.len() {
        //skip comments
        if decls[i].starts_with("//") || decls[i] == "" {
            continue;
        }
        
        if decls[i].len() != 0 {  
            if first_run {
            
                let component_decls = &decls[i];

                component_names = component_decls.split(" ").map(|s| s.into()).collect();

                if component_names[0] == "system"{
                    //do not include element 0 as that is the system keyword
                    for j in 1..component_names.len() {
                        let s = component_names[j].replace(",", "");
                        let s_cleaned = s.replace(";", "");
                        component_names[j] = s_cleaned.clone();
                        components.push(s_cleaned);
                    }
                    first_run = false;
                } else {
                    panic!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
                }
            }
                
            let split_string: Vec<String> = decls[i].split(" ").map(|s| s.into()).collect();
            if split_string[0].as_str() == "IO" {
                let component_name = split_string[1].clone();

                if component_names.contains(&component_name) {
                    for i in 2..split_string.len(){

                        let s = split_string[i].replace("{", "");
                        let p = s.replace("}", "");
                        let comp_actions : Vec<String> = p.split(",").map(|s| s.into()).collect();
                        for action in comp_actions {                        
                            if action.len() == 0 {
                                continue;
                            }
                            if action.ends_with("?") {
                                let r = action.replace("?", "");
                                if let Some(Channel_vec) = input_actions.get_mut(&component_name){
                                    Channel_vec.push(r)
                                } else {
                                    let mut Channel_vec = vec![];
                                    Channel_vec.push(r);
                                    input_actions.insert(component_name.clone(),Channel_vec);
                                }
                                
                            } else if action.ends_with("!") {
                                let r = action.replace("!", "");
                                if let Some(Channel_vec) = output_actions.get_mut(&component_name){
                                    Channel_vec.push(r.clone())
                                } else {
                                    let mut Channel_vec = vec![];
                                    Channel_vec.push(r.clone());
                                    output_actions.insert(component_name.clone(),Channel_vec);
                                }
                            } else {
                                panic!("Channel type not defined for Channel {:?}", action)
                            }
                        }
                        
                    }

                } else {
                    panic!("Was not able to finde component name: {:?} in declared component names: {:?}", component_name, component_names)
                }
            }

        }
    }
    Ok(SystemSpecification {
        components: components,
        input_actions: input_actions,
        output_actions: output_actions,
    })
}