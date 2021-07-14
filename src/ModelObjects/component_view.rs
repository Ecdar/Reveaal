use crate::ModelObjects::component::{
    Component, DeclarationProvider, Declarations, Location, LocationType,
};
use crate::ModelObjects::max_bounds::MaxBounds;

#[derive(Debug, Clone)]
pub struct ComponentView<'a> {
    component: &'a Component,
    declarations: Declarations,
    clock_index_offset: u32,
}

impl<'a> ComponentView<'a> {
    pub fn create(component: &'a Component, clock_index_offset: u32) -> Self {
        let mut declarations = component.get_declarations().clone();
        declarations.update_clock_indices(clock_index_offset);

        ComponentView {
            component,
            declarations,
            clock_index_offset,
        }
    }

    pub fn get_component(&self) -> &'a Component {
        self.component
    }

    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        self.component.get_max_bounds(dimensions)
    }

    pub fn get_name(&self) -> &String {
        self.component.get_name()
    }

    pub fn get_locations(&self) -> &Vec<Location> {
        self.component.get_locations()
    }

    pub fn get_initial_location(&self) -> &Location {
        self.component
            .get_locations()
            .iter()
            .find(|location| location.get_location_type() == &LocationType::Initial)
            .unwrap()
    }

    pub fn clock_count(&self) -> u32 {
        self.component.get_declarations().get_clocks().len() as u32
    }
}

impl<'a> DeclarationProvider for ComponentView<'a> {
    fn get_declarations(&self) -> &Declarations {
        &self.declarations
    }
    fn get_type(&self) -> &str {
        "View"
    }
}
