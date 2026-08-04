use crate::define_views;

define_views! {
    RootDataset {
        types: ["Dataset"],
        terms: ["hasPart", "mainEntity"],
    }

    CreateAction {
        types: ["CreateAction"],
        terms: ["instrument", "result"],
    }

    ControlAction {
        types: ["ControlAction"],
        terms: ["instrument", "object"],
    }

    OrganizeAction {
        types: ["OrganizeAction"],
        terms: ["instrument", "object", "result"],
    }

    Person {
        types: ["Person"],
        terms: ["name"],
    }

    Organization {
        types: ["Organization"],
        terms: ["name"],
    }

    ComputerLanguage {
        types: ["ComputerLanguage"],
        terms: ["name"],
    }

    FormalParameter {
        types: ["FormalParameter"],
        terms: ["additionalType"],
    }
    SoftwareApplication {
        types: ["SoftwareApplication"],
        terms: ["name"],
    }

    HowToStep {
        types: ["HowToStep"],
        terms: ["workExample"],
    }
}
