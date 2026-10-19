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
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


def ecosystem(here="quire-verification-contracts"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f

FIGURES = [hero, intake, ecosystem]
