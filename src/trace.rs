// SPDX-License-Identifier: AGPL-3.0-or-later
//! The walk event trace: the ordered enter and exit events of one walk,
//! with the size of the heap stack after each.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Write};
use core::ops::ControlFlow;

use crate::reservation::Unlimited;
use crate::walker::{drive, Probe, Walk};

/// The `schema` field of the JSON a [`Trace`] writes.
///
/// The version changes whenever a field is renamed, removed or reinterpreted.
pub const TRACE_SCHEMA: &str = "filament-walk.trace/v1";

/// Whether an [`Event`] entered or exited a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The node was entered; its children were named.
    Enter,
    /// The node was exited, after all its children.
    Exit,
}

/// One step of a traced walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Whether this step entered or exited a node.
    pub kind: Kind,
    /// The caller's label for the node.
    pub node: usize,
    /// The labels of the children the enter named, in entry order; empty for
    /// an exit.
    pub children: Vec<usize>,
    /// The tasks on the heap stack after the step: frames awaiting their exit
    /// plus children awaiting their enter.
    pub tasks: usize,
}

/// The ordered events of one walk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    /// The events, in the order the walk performed them.
    pub events: Vec<Event>,
}

impl Trace {
    /// Write the trace as JSON, one event per line.
    ///
    /// The record is `{"schema": "filament-walk.trace/v1", "events": [...]}`
    /// where each event is `{"event": "enter" | "exit", "node": <label>,
    /// "children": [<label>, ...], "tasks": <count>}`. `children` is empty
    /// for an exit. `tasks` is the heap stack size after the step. The text
    /// ends with a newline and is the same for the same walk.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        // Writing to a `String` cannot fail.
        let _ = self.write_json(&mut out);
        out
    }

    fn write_json(&self, out: &mut String) -> fmt::Result {
        writeln!(
            out,
            "{{\n  \"schema\": \"{TRACE_SCHEMA}\",\n  \"events\": ["
        )?;
        for (index, event) in self.events.iter().enumerate() {
            let name = match event.kind {
                Kind::Enter => "enter",
                Kind::Exit => "exit",
            };
            write!(
                out,
                "    {{\"event\": \"{name}\", \"node\": {}, \"children\": [",
                event.node
            )?;
            for (at, child) in event.children.iter().enumerate() {
                if at > 0 {
                    out.push_str(", ");
                }
                write!(out, "{child}")?;
            }
            let comma = if index + 1 < self.events.len() {
                ","
            } else {
                ""
            };
            writeln!(out, "], \"tasks\": {}}}{comma}", event.tasks)?;
        }
        out.push_str("  ]\n}\n");
        Ok(())
    }
}

/// Run `walker` over the tree below `root` as [`crate::walk`] does, and
/// record each enter and exit as an [`Event`].
///
/// `label` names a node for the record; an exit carries the label of the node
/// it leaves. A walk that ends early returns its [`ControlFlow`] and the
/// events up to that point.
///
/// # Panics
///
/// If the allocator cannot grow the heap stack.
pub fn walk_traced<W: Walk + ?Sized>(
    walker: &mut W,
    root: W::Node,
    label: impl FnMut(&W::Node) -> usize,
) -> (ControlFlow<W::Stop>, Trace) {
    let mut recorder = Recorder {
        label,
        open: Vec::new(),
        current: 0,
        children: Vec::new(),
        trace: Trace::default(),
    };
    let flow = match drive(walker, root, &mut Unlimited, &mut recorder) {
        Ok(flow) => flow,
        Err(refusal) => panic!("filament-walk: heap stack growth failed: {refusal:?}"),
    };
    (flow, recorder.trace)
}

struct Recorder<L> {
    label: L,
    /// Labels of the entered nodes not yet exited, innermost last.
    open: Vec<usize>,
    current: usize,
    children: Vec<usize>,
    trace: Trace,
}

impl<N, L: FnMut(&N) -> usize> Probe<N> for Recorder<L> {
    fn before_enter(&mut self, node: &N) {
        self.current = (self.label)(node);
    }

    fn entered(&mut self, children: &[N]) {
        self.children = children.iter().map(&mut self.label).collect();
    }

    fn stepped(&mut self, tasks: usize) {
        self.open.push(self.current);
        self.trace.events.push(Event {
            kind: Kind::Enter,
            node: self.current,
            children: core::mem::take(&mut self.children),
            tasks,
        });
    }

    fn exited(&mut self, tasks: usize) {
        let node = self.open.pop().unwrap_or(self.current);
        self.trace.events.push(Event {
            kind: Kind::Exit,
            node,
            children: Vec::new(),
            tasks,
        });
    }
}
