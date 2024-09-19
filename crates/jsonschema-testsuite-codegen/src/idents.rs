use std::collections::HashSet;

pub(crate) fn get_unique(base: &str, used: &mut HashSet<String>) -> String {
    let mut name = base.to_string();
    let mut counter = 1;

    while !used.insert(name.clone()) {
        name = format!("{}_{}", base, counter);
        counter += 1;
    }

    name
}
