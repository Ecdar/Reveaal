#[cfg(test)]
mod composition_tests {
    use crate::tests::refinement::Helper::json_run_query;
    use crate::System::executable_query::QueryResult;

    static PATH: &str = "samples/json/Conjunction";
    static ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn Test1Test1IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test1") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test2") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test3") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test4") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test5") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test1Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test1||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test2") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test3") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test4") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test5") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test2Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test2||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test3IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test3") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test4") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test5") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test3Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test3||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test4IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test4") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test5") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test4Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test4||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test5IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test5") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test5Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test5||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test6IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test6") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test6Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test6||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test7IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test7") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test7Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test7||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test8Test8IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8||Test8") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test8Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test8Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test8Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test8Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test8||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test9Test9IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9||Test9") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test9Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test9Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test9Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test9||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test10Test10IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10||Test10") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test10Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test10Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test10||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test11Test11IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test11||Test11") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test11Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test11||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Test12Test12IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(PATH, "consistency:Test12||Test12") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2Adm2IsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Adm2||Adm2") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2AdministrationIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||Administration")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2HalfAdm1IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||HalfAdm1")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2HalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||HalfAdm2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2MachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||Machine")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2Machine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2Machine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2ResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Adm2||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Adm2SpecIsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Adm2||Spec") {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationAdministrationIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Administration")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationHalfAdm1IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||HalfAdm1")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationHalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||HalfAdm2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Machine")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn AdministrationSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Administration||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1HalfAdm1IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||HalfAdm1")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1HalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||HalfAdm2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1MachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||Machine")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1Machine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1Machine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1ResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm1SpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm1||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2HalfAdm2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||HalfAdm2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2MachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||Machine")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2Machine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2Machine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2ResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn HalfAdm2SpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:HalfAdm2||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn MachineMachineIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine||Machine")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn MachineMachine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn MachineMachine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn MachineResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn MachineSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine2Machine2IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2||Machine2")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine2Machine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine2ResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine2SpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine2||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine3Machine3IsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3||Machine3")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine3ResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn Machine3SpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Machine3||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn ResearcherResearcherIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Researcher||Researcher")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn ResearcherSpecIsConsistent() {
        if let QueryResult::Consistency(res) =
            json_run_query(ECDAR_UNI, "consistency:Researcher||Spec")
        {
            assert!(res)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn SpecSpecIsConsistent() {
        if let QueryResult::Consistency(res) = json_run_query(ECDAR_UNI, "consistency:Spec||Spec") {
            assert!(res)
        } else {
            assert!(false)
        }
    }
}
