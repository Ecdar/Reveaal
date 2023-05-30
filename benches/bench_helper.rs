use reveaal::tests::TEST_SETTINGS;
use reveaal::{ComponentLoader, JsonProjectLoader};

const PATH: &str = "samples/json/EcdarUniversity";

static mut UNI_LOADER: Option<Box<dyn ComponentLoader>> = None;

pub fn get_loader() -> &'static mut Box<dyn ComponentLoader + 'static> {
    unsafe {
        match &mut UNI_LOADER {
            Some(ref mut l) => l,
            None => {
                UNI_LOADER = Some(load_everything());
                UNI_LOADER.as_mut().unwrap()
            }
        }
    }
}

fn load_everything() -> Box<dyn ComponentLoader> {
    let mut loader =
        JsonProjectLoader::new_loader(PATH.to_string(), TEST_SETTINGS).to_comp_loader();
    let _ = loader.get_component("Adm2");
    let _ = loader.get_component("Administration");
    let _ = loader.get_component("HalfAdm1");
    let _ = loader.get_component("HalfAdm2");
    let _ = loader.get_component("Machine");
    let _ = loader.get_component("Machine2");
    let _ = loader.get_component("Machine3");
    let _ = loader.get_component("Machine4");
    let _ = loader.get_component("Researcher");
    let _ = loader.get_component("Spec");
    loader
}
