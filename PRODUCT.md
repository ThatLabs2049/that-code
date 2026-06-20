# Muse — Product Specification

## 1. Vision

Muse is a **dual-agent AI companion system**. Users interact with a warm, emotionally intelligent character while a separate execution agent handles planning and task completion behind the scenes.

The goal is an AI that feels **personal and engaging** while remaining **highly capable and practical**.

Instead of exposing a task-oriented assistant directly, users communicate with a companion who understands intentions, provides emotional engagement, and transforms requests into structured plans for execution.

---

## 2. Core concept

### Agent 1 — Companion

The main user-facing character.

| Attribute | Description |
|-----------|-------------|
| Role | Personality layer |
| Temperature | High — creative, natural |
| Visibility | Always visible to the user |
| Tone | Warm, supportive, curious |

**Responsibilities:**

- Build rapport and make the user feel heard
- Understand goals and intentions through natural conversation
- Ask meaningful follow-up questions
- Convert vague requests into structured plans
- Decide whether a task requires execution
- Present executor results in the companion's voice

**Example:**

> **User:** "I want to start freelancing but I don't know where to begin."
>
> **Companion (Luna):** Discusses goals, asks clarifying questions, provides encouragement, builds a plan, then creates a structured task request for the executor.

### Agent 2 — Executor

The hidden task layer (optionally visible to power users).

| Attribute | Description |
|-----------|-------------|
| Role | Task and planning layer |
| Temperature | Low — deterministic reasoning |
| Visibility | Hidden by default; thoughts/actions optionally exposed |
| Tone | Neutral, structured, precise |

**Responsibilities:**

- Receive user context, companion analysis, and structured objectives
- Plan and execute tasks
- Use tools when available
- Return action plans, research, generated content, and tool results

**Receives:**

- User context
- Companion analysis
- Structured objectives

**Returns:**

- Action plans
- Research
- Generated content
- Tool results

---

## 3. User experience

### Flow

1. User opens the app.
2. Companion greets them.
3. User chats naturally.
4. Companion decides whether execution is required.

**When execution is needed:**

1. Companion analyzes the request.
2. Companion creates a structured objective.
3. Executor processes the task.
4. Result is returned to the companion.
5. Companion presents the final response naturally.

The user sees **one conversation thread** — not separate agent chats.

### Optional executor visibility

Users who want transparency can optionally view executor thinking and actions. This is **not** a full secondary chat interface (unlike Cursor's agent panel). It is a lightweight peek behind the curtain.

---

## 4. MVP scope (v1.0.0)

Scope is intentionally small.

### In scope

- Single personality (Luna)
- Single chat screen
- Local conversation history
- Agent orchestration (companion → executor pipeline)
- Basic planning pipeline
- Clean desktop UI
- OpenAI-compatible API integration

### Out of scope

- Voice
- Avatars
- Multiple personalities
- Memory graph / long-term memory
- Plugin marketplace
- Complex RAG
- Cloud sync

---

## 5. First personality — Luna

| Trait | Description |
|-------|-------------|
| Warm | Friendly, approachable, never cold or robotic |
| Supportive | Validates feelings, celebrates progress |
| Curious | Asks thoughtful follow-ups |
| Playful | Light humor when appropriate |
| Encouraging | Motivates without toxic positivity |

### Behavioral rules

- **Never** act as a generic assistant ("As an AI language model…")
- **Always** respond like a companion
- Ask meaningful follow-up questions
- Maintain emotional consistency across the conversation
- Prioritize user comfort and engagement
- Transform actionable requests into structured objectives for the executor

---

## 6. Success criteria (v1.0.0)

A user should be able to:

- [ ] Install the desktop app
- [ ] Chat with Luna
- [ ] Ask personal questions and receive companion-style responses
- [ ] Ask productivity questions and receive useful guidance
- [ ] Request plans and advice backed by the executor
- [ ] Experience the dual-agent architecture without managing two agents

The experience should feel like **talking to a companion**, not a traditional AI assistant.

---

## 7. Conversation types

| Type | Companion role | Executor involved? |
|------|----------------|-------------------|
| Casual chat | Full response | No |
| Emotional support | Empathy, reflection | Rarely |
| Advice / brainstorming | Discussion, questions | Sometimes |
| Plans, research, tasks | Intent capture, framing | Yes |
| Productivity requests | Clarify scope, encourage | Yes |

---

## 8. Product constraints

- **Local-first** — conversations stored in SQLite on device
- **Privacy-conscious** — API keys and history stay local unless user opts in to future sync
- **RTL-safe** — English-first; Persian and other RTL languages must render correctly
- **Offline-aware** — graceful degradation when API is unavailable (future enhancement)

---

## 9. Future versions (summary)

| Version | Focus |
|---------|-------|
| v1.1 | Multiple personalities |
| v1.2 | Long-term memory |
| v1.3 | Voice conversations |
| v1.4 | Local models |
| v2.0 | Multi-agent ecosystem, user-created personalities, advanced memory, tool marketplace |

See [ROADMAP.md](./ROADMAP.md) for detail.

---

## 10. Non-goals (v1)

- Replacing professional therapy or medical advice
- Real-time collaboration or multi-user chat
- Mobile apps (desktop-first for v1)
- Built-in web browsing without explicit user consent
- Autonomous background actions without user awareness

---

## 11. Onboarding

**Status:** Implemented in v2.2 — lightweight modal flow with skip.

1. Welcome + one-line value prop
2. API connection (test connection; Ollama without key)
3. Optional workspace folder picker
4. Personality picker (Luna, Sage, Spark)
5. Land in chat with personality-specific greeting

Keep onboarding under 60 seconds. Users can **Skip for now** without blocking chat.

---

## 12. Chat screen (v1)

### Layout

- **Header** — app name, settings access, optional executor visibility toggle
- **Message list** — scrollable conversation; companion messages only in default view
- **Composer** — text input, send button, optional attach (future)

### Message types (internal)

| Type | User-visible |
|------|--------------|
| User message | Yes |
| Companion message | Yes |
| Executor activity | Optional (collapsed by default) |
| System events | No (errors surfaced via companion) |

---

## 13. Settings (v1)

- API provider and key
- Model selection (companion vs executor, if separate)
- Executor visibility toggle
- Clear conversation history
- Theme (follows §16)

---

## 14. Error handling (UX)

Errors should never break the companion illusion abruptly.

| Scenario | UX |
|----------|-----|
| API unreachable | Luna explains gently; retry option |
| Executor failure | Luna summarizes what went wrong; offers alternatives |
| Rate limit | Luna asks user to wait or try later |
| Invalid API key | Clear settings prompt, not a raw error dump |

---

## 15. Accessibility

- Keyboard navigation for chat and settings
- Sufficient color contrast on dark base (see §16)
- Screen reader labels on interactive elements
- Respect `prefers-reduced-motion`
- RTL layout support for Persian and other RTL locales

---

## 16. Visual design

Muse should feel **calm, warm, and modern** — a quiet space for conversation, not a dashboard.

### Design principles

1. **Peaceful** — low visual noise, generous whitespace
2. **Warm** — soft accents, not sterile corporate blues
3. **Focused** — chat is the hero; chrome stays minimal
4. **Consistent** — spacing, typography, and motion follow a single system

### Color (dark-first)

| Token | Role | Guidance |
|-------|------|----------|
| `--bg-base` | App background | Deep neutral (not pure black) |
| `--bg-elevated` | Cards, composer | Slightly lighter surface |
| `--text-primary` | Body, messages | High contrast on base |
| `--text-muted` | Timestamps, hints | Readable but secondary |
| `--accent` | Send button, links, Luna highlights | Warm tone (amber, coral, or soft violet) |
| `--border-subtle` | Dividers | Low contrast |

Light theme is a v1 stretch goal; design dark-first.

### Typography

- **UI:** clean sans-serif (system stack or Inter-like)
- **Messages:** comfortable reading size (15–16px), relaxed line height
- **Headings:** minimal use; chat UI rarely needs large headings

### Spacing

- 4px base grid
- Message bubbles: consistent padding (12–16px)
- Composer: fixed bottom, safe area on all platforms

### Motion

- Subtle message appear (fade/slide, <200ms)
- No distracting loops or parallax
- Disable animations when `prefers-reduced-motion: reduce`

### Luna's presence (without avatars)

- Distinct companion message styling (soft accent border or subtle background)
- Optional name label "Luna" on first message in a session
- No avatar images in v1 — personality comes through language and tone

### Platform

- Native window chrome via Tauri
- Consistent look on Windows, macOS, and Linux
- Title bar integrates with app theme where OS allows

---

## 17. Open source

Muse is intended for GitHub as an open-source project.

- Clear documentation for contributors
- Transparent architecture and agent design
- Licensed under MIT (see [LICENSE](./LICENSE))
