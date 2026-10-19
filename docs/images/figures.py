# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-verification-contracts documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The numbers drawn come
from src/lib.rs (limits) and contracts/README.md (packet contents).
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """The vocabulary this crate owns and who reads which part of it."""
    f = Figure(
        "hero",
        980,
        406,
        "quire-verification-contracts owns five shared vocabularies: the five public verification "
        "contracts with their error codes, bounded JSON intake, RFC 8785 identity, the "
        "shared-reference packet and the checked-operation catalog. The private quire-verification "
        "re-exports all of it, quire-protocol uses the contracts directly, quire-contract-ir reads "
        "the catalog, and the packet is a temporary copy from the private quire-specification.",
    )
    rows = [
        ("E02 contracts", "5 wire types · error codes"),
        ("bounded JSON", "16 MiB · depth 128 · 100k items"),
        ("RFC 8785 identity", "canonical bytes · sha256-jcs"),
        ("shared-reference packet", "2 schemas · 14 fixtures"),
        ("operation catalog", "135 operations · one copy"),
    ]
    cx, cw, top, step = 330, 300, 34, 66
    f.rect(cx - 20, 0, cw + 40, 5 * step + 36, "faint", fill=False, rx=8)
    f.text(cx - 8, 20, "QUIRE-VERIFICATION-CONTRACTS", "muted", "mono", SMALL)
    for i, (t, s) in enumerate(rows):
        f.box(cx, top + i * step, cw, 56, "model", t, s)
    f.box(0, top + 3 * step, 220, 56, "ink", "quire-specification", "private · source of the copy", dashed=True)
    f.line([(220, top + 3 * step + 28), (cx - 3, top + 3 * step + 28)], "ink", dashed=True)
    f.note(8, top + 3 * step - 10, "data-only copy", "muted")
    consumers = [
        (0, "quire-verification", "private · re-exports all of it", True),
        (2, "quire-protocol", "private · uses the contracts", True),
        (4, "quire-contract-ir", "reads the catalog", False),
    ]
    for i, name, what, private in consumers:
        y = top + i * step
        f.line([(cx + cw, y + 28), (cx + cw + 60, y + 28)], "model")
        f.box(cx + cw + 63, y, 987 - (cx + cw + 63) - 7, 56, "ink", name, what, dashed=private)
    f.legend(0, 400, [("model", "vocabulary kept here", "box"), ("ink", "other repository", "box"), ("ink", "private", "dash")])
    return f


def intake():
    """What happens to bytes and to values at the boundary."""
    f = Figure(
        "intake",
        980,
        300,
        "JSON bytes pass parse_bounded_json, which refuses documents over 16 MiB, nesting over 128 "
        "levels or arrays over 100,000 items, then validate_contract checks the schema and returns "
        "one of five typed contracts; any failure is a stable error code. Separately, jcs_canonicalize "
        "turns a value into RFC 8785 bytes and jcs_sha256 into a sha256-jcs digest.",
    )
    f.text(0, 14, "INTAKE", "muted", "mono", SMALL)
    f.box(0, 30, 150, 56, "ink", "JSON bytes", "from a caller")
    f.box(200, 30, 210, 56, "logic", "parse_bounded_json", "size · depth · array")
    f.box(460, 30, 190, 56, "logic", "validate_contract", "draft 2020-12 schema")
    f.box(700, 30, 280, 56, "model", "PublicContract", "one of five typed contracts")
    f.line([(150, 58), (197, 58)], "ink")
    f.line([(410, 58), (457, 58)], "logic")
    f.line([(650, 58), (697, 58)], "logic")
    f.box(200, 130, 450, 50, "evidence", "VerificationError", "a stable code, never a panic")
    f.line([(305, 86), (305, 127)], "evidence")
    f.line([(555, 86), (555, 127)], "evidence")
    f.text(0, 214, "IDENTITY", "muted", "mono", SMALL)
    f.box(0, 230, 150, 56, "ink", "any JSON value", "key order varies")
    f.box(200, 230, 210, 56, "logic", "jcs_canonicalize", "RFC 8785 bytes")
    f.box(460, 230, 190, 56, "logic", "jcs_sha256", "hash of the bytes")
    f.box(700, 230, 280, 56, "model", "sha256-jcs:<hex>", "same value, same digest")
    f.line([(150, 258), (197, 258)], "ink")
    f.line([(410, 258), (457, 258)], "logic")
    f.line([(650, 258), (697, 258)], "logic")
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("ink", "private", "dash"),
                      ("model", "feeds", "fill")])
    return f


def ecosystem_here():
    return ecosystem(here="quire-verification-contracts")


FIGURES = [hero, intake, ecosystem_here]
