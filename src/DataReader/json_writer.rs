use crate::ModelObjects::component::Automaton;
use std::fs::File;

pub fn automaton_to_json_file(project_path: &str, automaton: &Automaton) {
    let path = format!(
        "{0}{1}Components{1}{2}.json",
        project_path,
        std::path::MAIN_SEPARATOR,
        automaton.get_name()
    );
    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, automaton).expect("Failed to serialize automaton");
}

pub fn automaton_to_json(automaton: &Automaton) -> String {
    serde_json::to_string(automaton).unwrap()
}
