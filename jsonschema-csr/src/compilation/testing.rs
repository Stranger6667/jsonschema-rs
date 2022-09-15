use crate::compilation::ValueReference;
use std::collections::HashMap;

/// Ensure every concrete reference is unique.
pub(crate) fn assert_concrete_references(values: &[ValueReference]) {
    let mut seen = HashMap::new();
    for (index, value) in values.iter().enumerate() {
        if let ValueReference::Concrete(reference) = value {
            if let Some(existing_index) = seen.insert(*reference as *const _, index) {
                panic!(
                    "Concrete reference `{}` at index {} was already seen at index {}",
                    reference, index, existing_index
                )
            }
        }
    }
}

/// Ensure every virtual reference points to a concrete one.
pub(crate) fn assert_virtual_references(values: &[ValueReference]) {
    'outer: for (reference_index, value) in values.iter().enumerate() {
        if let ValueReference::Virtual(reference) = value {
            for (target_index, target) in values.iter().enumerate() {
                if let ValueReference::Concrete(target) = target {
                    println!(
                        "Compare\n  `{}` ({:p}) at {} vs `{}` ({:p}) at {}",
                        reference,
                        *reference as *const _,
                        reference_index,
                        target,
                        *target as *const _,
                        target_index
                    );
                    if std::ptr::eq(*reference, *target) {
                        // Found! Check the next one
                        println!(
                            "Found for `{}` ({:p}) at {}",
                            reference, *reference as *const _, reference_index
                        );
                        continue 'outer;
                    }
                }
            }
            panic!(
                "Failed to find a concrete reference for a virtual reference `{}` at index {}",
                reference, reference_index
            )
        }
    }
}

/// Display value references in a slice.
pub(crate) fn print_values(values: &[ValueReference]) {
    for (id, value) in values.iter().enumerate() {
        match value {
            ValueReference::Concrete(reference) => {
                println!("C[{}]: {}", id, reference)
            }
            ValueReference::Virtual(reference) => {
                println!("V[{}]: {}", id, reference)
            }
        }
    }
}
