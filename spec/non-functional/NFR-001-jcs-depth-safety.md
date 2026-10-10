---
id: NFR-001
title: "Keep dynamic-depth canonicalization off the native traversal stack"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-verification-contracts/FR-001
    type: constrains
---
# NFR-001: Keep dynamic-depth canonicalization off the native traversal stack

## Statement

When a public jcs helper encodes data whose depth follows its input, QVC SHALL use an authoritative or consumer-owned explicit heap traversal stack without adding a depth cap.

## Scope

Applies to the narrowed inputs in [FR-001](../functional/FR-001-authoritative-jcs-encode-contract.md).
FixedShape types retain native serde traversal whose depth is fixed by the
schema. Genuine derivations compute depth from every field type; hand-written
implementations have the same obligation. Serialization overrides are
admissible only when their emitted depth is also fixed. Recursive foreign
values cannot become FixedShape through a literal depth or blanket wrapper.

`Encode` is an implementation contract, not a compiler proof that custom
consumer code is iterative. QVC cannot guarantee safety for a hand-written
Encode implementation that violates that contract. Review must examine
dynamic-depth implementations, not infer safety from the trait bound alone.

## Rationale

An unrestricted Serialize implementation controls recursive calls into the
serializer. Existing canonical policy permits only fixed-depth serde and
uses explicit-stack events for input-dependent depth. Narrowing the public
contract preserves that distinction instead of hiding a native-stack risk,
introducing a depth refusal, or pretending the writer can suspend arbitrary
caller-owned recursive execution.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Value and custom event-source array nesting encoded on a 512 KiB native stack | 100000 levels | Complete literal canonical bytes without native stack overflow | Test |
| Output agreement on 512 KiB and 8 MiB native stacks | Identical bytes and content digests | Zero differences | Test |
| New depth-policy or ingestion-policy refusals | None | Zero added caps or inherited ingestion limits in jcs helpers | Inspection |

## Verification

Construct an authoritative Value and a consumer-owned explicit-stack event
source representing 100000 nested arrays around null. The byte oracle is
100000 opening brackets, `null`, and 100000 closing brackets. Canonicalize,
compare, and hash on threads with 512 KiB and 8 MiB stacks; verify bytes and
independent content-only digest agree. Build each operand without recursive
helpers. Dispose of deep Value through authoritative `drop_value`, so
serde_json's recursive Drop is not confused with encoder traversal.

Also exercise a dynamic-depth object chain, verify byte accounting against
an explicit matching authoritative byte limit, and verify one byte less
refuses at the authoritative API. This limit-boundary probe is not a new
parameter or policy limit on QVC's public helpers. Review source to confirm
that every dynamic-depth source uses an explicit traversal stack and no
stack-reset, hidden depth ceiling, or lossy conversion appears.

The measurements are prospective acceptance checks. This specification
checkpoint has not run them or claimed runtime safety for arbitrary custom
Serialize/Encode implementations.
