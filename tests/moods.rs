//! Every config choice the schema offers must be catalogued by mood.
use std::collections::BTreeSet;

use bgsvg::{mood::MOODS, params::DESCRIPTOR};
use prost::Message;
use prost_types::FileDescriptorSet;

/// The messages whose oneof branches are config choices.
const CHOICE_HOLDERS: [&str; 3] = ["Background", "Icon", "Overlay"];

/// Dotted paths of every choice in the schema: each real oneof branch of
/// the `CHOICE_HOLDERS`, and each top-level mood block.
fn schema_choices() -> BTreeSet<String> {
    let set = FileDescriptorSet::decode(DESCRIPTOR).unwrap();
    let mut out = BTreeSet::new();
    for msg in set.file.iter().flat_map(|f| &f.message_type) {
        match msg.name() {
            name if CHOICE_HOLDERS.contains(&name) => {
                let name = name.to_lowercase();
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

/// A renamed message would drop out of the walk and leave the coverage test passing on nothing.
#[test]
fn the_walk_finds_every_message_it_reads() {
    let set = FileDescriptorSet::decode(DESCRIPTOR).unwrap();
    let names: BTreeSet<_> = set
        .file
        .iter()
        .flat_map(|f| &f.message_type)
        .map(|m| m.name())
        .collect();
    for want in CHOICE_HOLDERS.iter().chain(&["Parameters"]) {
        assert!(names.contains(want), "no message {want} in the schema");
    }
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
