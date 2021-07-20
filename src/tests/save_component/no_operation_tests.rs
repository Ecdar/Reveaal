#[cfg(test)]
mod no_operation_tests {
    use crate::tests::save_component::save_comp_helper::save_comp_helper::json_reconstructed_component_refines_base_self;

    static PATH: &str = "samples/json/Conjunction";
    static ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn T1SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test1");
    }
    #[test]
    fn T2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test2");
    }
    #[test]
    fn T3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test3");
    }
    #[test]
    fn T4SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test4");
    }
    #[test]
    fn T5SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test5");
    }
    #[test]
    fn T6SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test6");
    }
    #[test]
    fn T7SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test7");
    }
    #[test]
    fn T8SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test8");
    }
    #[test]
    fn T9SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test9");
    }
    #[test]
    fn T10SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test10");
    }
    #[test]
    fn T11SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test11");
    }
    #[test]
    fn T12SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(PATH, "Test12");
    }

    #[test]
    fn Adm2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2");
    }

    #[test]
    fn AdministrationSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration");
    }

    #[test]
    fn HalfAdm1SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1");
    }

    #[test]
    fn HalfAdm2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2");
    }

    #[test]
    fn MachineSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine");
    }

    #[test]
    fn Machine2SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2");
    }

    #[test]
    fn Machine3SavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3");
    }

    #[test]
    fn ResearcherSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher");
    }

    #[test]
    fn SpecSavedRefinesSelf() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Spec");
    }
}
