# Muse — Agent Specifications

Muse uses two cooperating agents. This document defines their roles, interfaces, and behavioral contracts.

---

## Overview

| | Companion (Agent 1) | Executor (Agent 2) |
|---|---------------------|---------------------|
| **Codename** | Personality layer | Task layer |
| **User-facing** | Yes — always | No — hidden by default |
| **Temperature** | High (e.g. 0.8–1.0) | Low (e.g. 0.2–0.4) |
| **Primary goal** | Connection, clarity, encouragement | Accurate plans, research, execution |
| **v1 persona** | Luna | None (neutral) |

---

## Companion Agent

### Purpose

The companion is the **only voice the user should hear** in normal use. It builds rapport, understands intent, and decides when to delegate work to the executor.

### Responsibilities

1. Greet and maintain ongoing conversation
2. Provide emotional intelligence and encouragement
3. Ask clarifying questions when intent is vague
4. Classify whether a message requires execution
5. Produce structured `TaskSpec` when execution is needed
6. Format executor results into natural companion language
7. Never break character as a generic AI assistant

### System prompt principles (Luna, v1)

Include in the companion system prompt:

- Identity: Luna — warm, supportive, curious, playful, encouraging
- Never say "As an AI…" or act as a generic assistant
- Prioritize user comfort; ask meaningful follow-ups
- When a task needs deep planning or research, emit a structured task request (see TaskSpec)
- When presenting executor results, weave them into conversational prose — not bullet dumps unless the user asked for a list

### Output modes

The companion returns one of two shapes:

#### Mode A — Direct response

No executor needed. Examples: casual chat, simple advice, emotional support.

```json
{
  "mode": "direct",
  "message": "That sounds like a big step — and honestly, a exciting one. What's drawing you to freelancing right now?"
}
```

#### Mode B — Delegate to executor

Execution needed. Companion may include a brief in-character holding message plus a task spec.

```json
{
  "mode": "delegate",
  "message": "Let me think through this properly for you — give me a moment.",
  "task_spec": {
    "objective": "Create a beginner freelancing starter plan",
    "context": "User wants to start freelancing but doesn't know where to begin",
    "constraints": ["actionable", "30-day horizon", "beginner-friendly"],
    "expected_output": "step-by-step plan with weekly milestones"
  }
}
```

The `message` in delegate mode can be shown immediately while the executor runs; the final user-visible message comes from the format pass.

### Format pass

After executor completion, the companion receives:

- Original user message
- Conversation context
- `TaskSpec`
- Executor result

It produces the **final** user-facing message in Luna's voice.

---

## Executor Agent

### Purpose

The executor performs **deterministic, task-focused work** — planning, structured analysis, research synthesis, and **sandboxed workspace tools** (v1.5+). It does not engage in small talk.

### Responsibilities

1. Accept `TaskSpec` and relevant context
2. Run a bounded tool loop (`list_dir`, `read_file`, `write_file`, `search_files`) inside the user-selected workspace
3. Produce structured, actionable output via `final_answer` JSON
4. Minimize hallucination; use tools instead of inventing file contents
5. Return machine-parseable results for the companion format pass (or template wrap when output is simple)
6. Log real tool steps in `activity_log` for optional user visibility

### Workspace tools (v1.5)

| Tool | Scope |
|------|--------|
| `list_dir` | List entries under a relative path |
| `read_file` | Read text files up to 512 KB |
| `write_file` | Create/overwrite files (overwrite gated by Settings) |
| `search_files` | Filename/content search within workspace |

All paths are relative to the workspace root configured in Settings. Requests outside the sandbox return tool errors logged in `activity_log`.

### Agent loop contract

Each executor turn is one JSON object:

```json
{"action":"tool_call","tool":"list_dir","arguments":{"path":"src"}}
```

or

```json
{"action":"final_answer","status":"success","summary":"...","content":"..."}
```

Maximum **10 steps** per delegated task.

### System prompt principles

- Neutral, precise, task-focused tone
- No personality flourishes
- Structured output preferred (JSON or markdown sections)
- State assumptions explicitly
- If information is insufficient, return `status: "needs_clarification"` with specific questions (routed back through companion)

### Input

```json
{
  "task_spec": {
    "objective": "...",
    "context": "...",
    "constraints": ["..."],
    "expected_output": "..."
  },
  "user_context": {
    "recent_messages": ["..."],
    "locale": "en"
  }
}
```

### Output

```json
{
  "status": "success",
  "summary": "One-paragraph executive summary for companion formatting",
  "content": "Detailed plan, research, or generated content",
  "structured": {},
  "activity_log": [
    { "step": "analyze_objective", "detail": "..." },
    { "step": "generate_plan", "detail": "..." }
  ]
}
```

| `status` | Meaning |
|----------|---------|
| `success` | Task completed; companion formats result |
| `needs_clarification` | Missing info; companion asks user |
| `error` | Failure; companion explains gracefully |

---

## TaskSpec schema

Canonical structure between companion and executor.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `objective` | string | yes | What to accomplish |
| `context` | string | yes | Relevant background from conversation |
| `constraints` | string[] | no | Limits, preferences, format requirements |
| `expected_output` | string | yes | What the executor should produce |

Validation happens in Rust before executor invocation.

---

## Orchestration sequence

```
1. User sends message
2. COMPANION (intent pass)
   └─ direct → save + return message
   └─ delegate → save holding message + TaskSpec
3. EXECUTOR (if delegate)
   └─ run with TaskSpec
   └─ save executor_run record
4. COMPANION (format pass)
   └─ produce final message
5. Save companion message + return to UI
```

---

## Intent classification

The companion decides delegation. Guidelines:

| User intent | Delegate? |
|-------------|-----------|
| "How are you?" | No |
| "I'm feeling stressed" | No (support first) |
| "Help me plan my week" | Yes |
| "Research X for me" | Yes |
| "What should I do about Y?" (vague) | No — clarify first, then maybe yes |
| "Give me a 30-day freelancing plan" | Yes |

When uncertain, the companion should **clarify before delegating** to avoid unnecessary executor calls.

---

## Model configuration (v1)

| Agent | Suggested temperature | Notes |
|-------|----------------------|-------|
| Companion | 0.8 – 1.0 | Personality and natural language |
| Executor | 0.2 – 0.4 | Consistency and structure |

Models may be the same endpoint with different parameters, or different models entirely (user-configurable in settings).

---

## Executor visibility (optional UX)

When enabled, the UI shows `activity_log` and optionally raw executor output in a collapsible panel. This is **supplementary** — not a second chat.

---

## Future extensions

| Version | Extension |
|---------|-----------|
| v1.1 | Per-personality companion prompts |
| v1.2 | Executor access to memory graph |
| v1.3 | Voice-specific companion prompts |
| v1.4 | Local model profiles |
| v2.0 | Multiple executors, tool marketplace |

---

## Prompt storage

- v1: prompts live in `src-tauri/src/ai/prompts/` as templates
- Version prompts with the app; document changes in [DECISIONS.md](./DECISIONS.md)
- Never hardcode API keys in prompts or source

---

## Related documents

- [ARCHITECTURE.md](./ARCHITECTURE.md) — pipeline and data model
- [PRODUCT.md](./PRODUCT.md) — Luna personality and UX
- [DECISIONS.md](./DECISIONS.md) — dual-agent rationale
