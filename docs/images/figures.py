# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""filament-walk's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Only the walk-order figure is defined here, because it uses the `columns` composite that
ixd-figures does not yet draw. The other figures are defined in docs/images/figures.json and
rendered by `make figures`.

Add --check to confirm the committed SVGs are current. The step sequence and
stack sizes in the walk figure are the trace of the example in src/lib.rs.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# ---- figures -----------------------------------------------------------------


def order():
    """The enter and exit order of the lib.rs example, with the stack size."""
    steps = [
        ("enter 0", "push 1, 2", 3),
        ("enter 1", "leaf", 3),
        ("exit 1", "frame back", 2),
        ("enter 2", "push 3", 3),
        ("enter 3", "leaf", 3),
        ("exit 3", "frame back", 2),
        ("exit 2", "frame back", 1),
        ("exit 0", "frame back", 0),
    ]
    f = Figure(
        "order",
        980,
        330,
        "A walk of the tree where node 0 has children 1 and 2 and node 2 has child 3 enters "
        "0, 1, 2, 3 and exits 1, 3, 2, 0: each node is entered before its children and exited "
        "after all of them. The heap stack holds at most three tasks.",
    )
    f.text(0, 12, "TREE: 0 → (1, 2) · 2 → (3)", "muted", "mono", SMALL)
    w, gap = 110, 12
    for i, (title, sub, _) in enumerate(steps):
        x = i * (w + gap)
        role = "logic" if title.startswith("enter") else "model"
        f.box(x, 28, w, 56, role, title, sub)
    f.text(0, 124, "TASKS ON THE HEAP STACK AFTER EACH STEP", "muted", "mono", SMALL)
    f.columns(
        0,
        134,
        980,
        180,
        [n for _, _, n in steps],
        labels=[t for t, _, _ in steps],
        ticks=[0, 1, 2, 3],
        role=["logic" if t.startswith("enter") else "model" for t, _, _ in steps],
    )
    return f

FIGURES = [order]
