use crate::define_views;

define_views! {
    TestSuite {
        types: ["TestSuite"],
        terms: ["TestSuite", "instance"],
    }

    TestInstance {
        types: ["TestInstance"],
        terms: ["TestInstance", "runsOn", "resource"],
    }

    TestService {
        types: ["TestService"],
        terms: ["TestService"],
    }
}
