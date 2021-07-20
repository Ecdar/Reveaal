#[cfg(test)]
mod Conjunction_tests {
    use crate::tests::refinement::Helper::json_run_query;
    use crate::System::executable_query::QueryResult;

    static PATH: &str = "samples/json/Conjunction";
    static ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn Adm2AndAdm2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Adm2 && Adm2")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndAdministrationIsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && Administration")
        {
            assert!(!res)
        }
    }

    #[test]
    fn Adm2AndHalfAdm1IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && HalfAdm1")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndHalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && HalfAdm2")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && Machine")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2 && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn Adm2AndSpecIsNotConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Adm2 && Spec")
        {
            assert!(!res)
        }
    }

    #[test]
    fn AdministrationAndAdministrationIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Administration")
        {
            assert!(res)
        }
    }

    #[test]
    fn AdministrationAndHalfAdm1IsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && HalfAdm1")
        {
            assert!(!res)
        }
    }

    #[test]
    fn AdministrationAndHalfAdm2IsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && HalfAdm2")
        {
            assert!(!res)
        }
    }

    #[test]
    fn AdministrationAndMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Machine")
        {
            assert!(res)
        }
    }

    #[test]
    fn AdministrationAndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn AdministrationAndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn AdministrationAndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn AdministrationAndSpecIsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration && Spec")
        {
            assert!(!res)
        }
    }

    #[test]
    fn HalfAdm1AndHalfAdm1IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && HalfAdm1")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndHalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && HalfAdm2")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && Machine")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm1AndSpecIsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1 && Spec")
        {
            assert!(!res)
        }
    }

    #[test]
    fn HalfAdm2AndHalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && HalfAdm2")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm2AndMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && Machine")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm2AndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm2AndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm2AndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn HalfAdm2AndSpecIsNotConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2 && Spec")
        {
            assert!(!res)
        }
    }

    #[test]
    fn MachineAndMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine && Machine")
        {
            assert!(res)
        }
    }

    #[test]
    fn MachineAndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn MachineAndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn MachineAndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn MachineAndSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine && Spec")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine2AndMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2 && Machine2")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine2AndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2 && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine2AndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2 && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine2AndSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2 && Spec")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine3AndMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3 && Machine3")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine3AndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3 && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn Machine3AndSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3 && Spec")
        {
            assert!(res)
        }
    }

    #[test]
    fn ResearcherAndResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Researcher && Researcher")
        {
            assert!(res)
        }
    }

    #[test]
    fn ResearcherAndSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Researcher && Spec")
        {
            assert!(res)
        }
    }

    #[test]
    fn SpecAndSpecIsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Spec && Spec")
        {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest1IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test1") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test2") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test3") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test4") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test5") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test1AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test2") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test3") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test4") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test5") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test2AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test3") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test4") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test5") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test3AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test4") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test5") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test4AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test5") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test5AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test6") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test6AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test7") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test7AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test8AndTest8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8&&Test8") {
            assert!(res)
        }
    }

    #[test]
    fn Test8AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test8AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test8AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test8AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test9AndTest9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9&&Test9") {
            assert!(res)
        }
    }

    #[test]
    fn Test9AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test9AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test9AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test10AndTest10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10&&Test10") {
            assert!(res)
        }
    }

    #[test]
    fn Test10AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test10AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test11AndTest11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test11&&Test11") {
            assert!(res)
        }
    }

    #[test]
    fn Test11AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test11&&Test12") {
            assert!(res)
        }
    }

    #[test]
    fn Test12AndTest12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test12&&Test12") {
            assert!(res)
        }
    }
}
