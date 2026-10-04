# AGENT.md — Agent Routing

This project defines four agents under `.claude/agents/`. Claude Code should
pick the agent that matches the task at hand instead of acting generically.

| Agent file | Use when the task is about... |
|---|---|
| `architect.md` | Deciding where new code belongs, defining traits/module boundaries, reviewing whether a change respects Clean Architecture + SOLID. |
| `rust-developer.md` | Writing or modifying Rust implementation code inside an already-decided module. |
| `code-reviewer.md` | Reviewing a diff/PR before it's considered done: SOLID compliance, clippy warnings, error handling, naming. |
| `test-engineer.md` | Writing unit/integration tests, designing fakes/mocks for `DiagnosticCheck` and `Reporter`. |

## Default flow for a feature request

1. **architect** decides the shape: which trait, which module, which
   pattern (Strategy for checks, Factory for construction, Adapter for
   wrapping `systemctl`/`sysinfo`, etc.).
2. **rust-developer** implements it inside the boundaries the architect set.
3. **test-engineer** adds tests that isolate the new code from real system
   state.
4. **code-reviewer** checks the result against `.claude/skills/` before it
   is considered finished.

A single Claude Code session can move through all four roles sequentially;
they don't need to be separate invocations, but the mindset switch should be
explicit — announce which "hat" is on when moving to the next step.
