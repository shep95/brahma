# implementation decisions

## 2026-10-10: smart object contents identity (#2163)

smart object instances carry a document-scoped contents id separately from their layer id. normal duplicates preserve it; new smart object via copy creates a new one. contents edits update the matching instances together, retaining their individual placement and filters. copies into another document remap contents ids as a batch, preserving sharing within the copied layers without joining existing destination objects.

native files persist this id. legacy embedded objects without an id remain independent: identical file bytes do not prove that two objects were intended to share edits. legacy nonempty linked paths and PSD source identifiers retain their explicit sharing relationship. PSD export keys embedded sources by contents identity as well as bytes, so independent equal-content objects stay independent.
