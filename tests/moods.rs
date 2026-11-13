//! Every config choice the schema offers must be catalogued by mood.
use std::collections::BTreeSet;

use bgsvg::{mood::MOODS, params::DESCRIPTOR};
use prost::Message;
use prost_types::FileDescriptorSet;

/// Dotted paths of every choice in the schema: each real oneof branch of
/// Background/Icon/Overlay, and each top-level mood block.
fn schema_choices() -> BTreeSet<String> {
    let set = FileDescriptorSet::decode(DESCRIPTOR).unwrap();
    let mut out = BTreeSet::new();
    for msg in set.file.iter().flat_map(|f| &f.message_type) {
        match msg.name() {
            "Background" | "Icon" | "Overlay" => {
                let name = msg.name().to_lowercase();
                for f in &msg.field {
                    // proto3 `optional` is a synthetic oneof, not a choice
                    if f.oneof_index.is_some() && !f.proto3_optional() {
                        out.insert(format!("{name}.{}", f.name()));
                    }
                }
            }
            "Parameters" => {
                for f in &msg.field {
                    if !["seed", "output", "background", "icon", "overlay"].contains(&f.name()) {
                        out.insert(f.name().to_string());
                    }
                }
            }
            _ => {}
        }
    }
    out
}

#[test]
fn every_choice_has_a_mood() {
    let keys: BTreeSet<_> = MOODS.iter().map(|(k, _)| k.to_string()).collect();
    let missing: Vec<_> = schema_choices().difference(&keys).cloned().collect();
    assert!(missing.is_empty(), "no mood for: {missing:?}");
}

#[test]
fn every_mood_entry_names_a_real_choice() {
    let choices = schema_choices();
    let stale: Vec<_> = MOODS
        .iter()
        .map(|(k, _)| *k)
        .filter(|k| !choices.contains(*k))
        .collect();
    assert!(stale.is_empty(), "not in the schema: {stale:?}");
}

#[test]
fn every_entry_has_at_least_one_mood() {
    assert!(MOODS.iter().all(|(_, m)| !m.is_empty()));
}
