// SPDX-License-Identifier: AGPL-3.0-or-later
//! The walk event-trace record: its fields, and the committed record of the
//! `src/lib.rs` example that the `order` figure reads.

use core::ops::ControlFlow;
use std::path::Path;

use filament_walk::{walk_traced, Children, Kind, Trace, Walk, TRACE_SCHEMA};

/// The committed trace of the `src/lib.rs` example tree.
const RECORD: &str = "docs/trace/lib-example.json";

struct Tree(Vec<Vec<usize>>);

/// The `src/lib.rs` example walk, with nothing to count.
struct Visit<'a>(&'a Tree);

impl Walk for Visit<'_> {
    type Node = usize;
    type Frame = usize;
    type Stop = ();

    fn enter(&mut self, node: usize, children: &mut Children<'_, usize>) -> ControlFlow<(), usize> {
        children.extend(self.0 .0[node].iter().copied());
        ControlFlow::Continue(node)
    }

    fn exit(&mut self, _frame: usize) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

/// The trace of the tree from the `src/lib.rs` example.
fn lib_example() -> Trace {
    let tree = Tree(vec![vec![1, 2], vec![], vec![3], vec![]]);
    let (flow, trace) = walk_traced(&mut Visit(&tree), 0, |node| *node);
    assert_eq!(flow, ControlFlow::Continue(()));
    trace
}

#[test]
fn lib_example_trace_matches_the_committed_record() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(RECORD);
    let fresh = lib_example().to_json();
    if std::env::var_os("FILAMENT_WALK_REGENERATE").is_some() {
        std::fs::write(&path, &fresh).expect("the record is writable");
    }
    let committed = std::fs::read_to_string(&path).expect("the record is committed");
    assert_eq!(
        committed, fresh,
        "{RECORD} is stale; run `make trace` and commit the result"
    );
}

#[test]
fn trace_orders_enters_and_exits_with_task_counts() {
    let events = lib_example().events;
    let steps: Vec<(Kind, usize, usize)> = events
        .iter()
        .map(|event| (event.kind, event.node, event.tasks))
        .collect();
    use Kind::{Enter, Exit};
    assert_eq!(
        steps,
        [
            (Enter, 0, 3),
            (Enter, 1, 3),
            (Exit, 1, 2),
            (Enter, 2, 3),
            (Enter, 3, 3),
            (Exit, 3, 2),
            (Exit, 2, 1),
            (Exit, 0, 0),
        ]
    );
    assert_eq!(events[0].children, [1, 2]);
    assert!(events[2].children.is_empty());
}

/// The record's field list is the contract the figure reads; a removed or
/// renamed field fails here.
#[test]
fn trace_record_field_list_is_versioned_and_fixed() {
    let record: serde_json::Value = serde_json::from_str(&lib_example().to_json()).expect("JSON");
    assert_eq!(record["schema"], "filament-walk.trace/v1");
    assert_eq!(TRACE_SCHEMA, "filament-walk.trace/v1");
    let mut keys: Vec<&str> = record
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["events", "schema"]);
    for event in record["events"].as_array().expect("an event list") {
        let mut keys: Vec<&str> = event
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["children", "event", "node", "tasks"]);
        assert!(matches!(event["event"].as_str(), Some("enter" | "exit")));
        assert!(event["node"].is_u64() && event["tasks"].is_u64() && event["children"].is_array());
    }
}
