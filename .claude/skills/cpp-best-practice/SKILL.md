---
name: cpp-best-practice
description: Use whenever C++ source code is written, read, or reviewed, in any context — not limited to learning sessions. Surfaces violations of modern C++ idioms and Core-Guidelines-style practices as concise, severity-labeled findings for the calling context to act on — teach-programming turns them into hints during learning sessions, otherwise state them directly. Do not rewrite the user's code unless explicitly asked for a fix.
---

# C++ Best Practices

## Stance
- Name the violated idiom or guideline, not a rewrite.

## Checklist
- Ownership: RAII by default; no raw `new`/`delete`; `unique_ptr` for exclusive ownership, `shared_ptr` only when sharing is real.
- Special members: Rule of Zero — hand-write them only when directly owning a resource.
- Const-correctness: `const` on unchanging state; `const&` or by-value for cheap, read-only params.
- Error handling: exception-safety guarantee matched to the operation; no throwing destructors.
- Containers/algorithms: `<algorithm>` over hand-rolled loops; `reserve`/`emplace` over `push_back` when size is known.
- Undefined behavior: dangling references, signed overflow, out-of-bounds access — hard stop, not a style note.
- Concurrency: `lock_guard`/`unique_lock` over manual `lock`/`unlock`; threads joined or detached explicitly, never left dangling.

## Exceptions
- Full corrected implementation explicitly requested: comply.
- `[Style]`-only nitpick in throwaway/experimental code: mention once, don't repeat.

## Guardrail
- No silent rewrites.

## Output
- Prefix every finding with one severity label: `[Style]`, `[Improvement]`, or `[Critical]`.
- `[Style]`: preference only, no functional effect.
- `[Improvement]`: idiom/robustness gap, not urgent.
- `[Critical]`: UB, memory-safety, or correctness risk — always surface, regardless of context.
