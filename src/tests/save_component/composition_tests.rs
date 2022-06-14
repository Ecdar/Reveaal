#[cfg(test)]
mod composition_tests {
    use crate::tests::save_component::save_comp_helper::save_comp_helper::json_reconstructed_component_refines_base_self;

    static PATH: &str = "samples/json/Conjunction";
    static ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn Adm2MachineSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine");
    }

    #[test]
    fn Adm2Machine2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine2");
    }

    #[test]
    fn Adm2Machine3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine3");
    }

    #[test]
    fn Adm2ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Researcher");
    }

    #[test]
    fn AdministrationMachineSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine");
    }

    #[test]
    fn AdministrationMachine2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine2");
    }

    #[test]
    fn AdministrationMachine3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine3");
    }

    #[test]
    fn AdministrationResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Researcher");
    }

    #[test]
    fn HalfAdm1MachineSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine");
    }

    #[test]
    fn HalfAdm1Machine2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine2");
    }

    #[test]
    fn HalfAdm1Machine3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine3");
    }

    #[test]
    fn HalfAdm1ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Researcher");
    }

    #[test]
    fn HalfAdm2MachineSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine");
    }

    #[test]
    fn HalfAdm2Machine2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine2");
    }

    #[test]
    fn HalfAdm2Machine3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine3");
    }

    #[test]
    fn HalfAdm2ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Researcher");
    }

    #[test]
    fn MachineResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Researcher");
    }

    #[test]
    fn MachineSpecSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Spec");
    }

    #[test]
    fn Machine2ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Researcher");
    }

    #[test]
    fn Machine2SpecSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Spec");
    }

    #[test]
    fn Machine3ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Researcher");
    }

    #[test]
    fn Machine3SpecSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Spec");
    }

    #[test]
    fn ResearcherSpecSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher || Spec");
    }
}
