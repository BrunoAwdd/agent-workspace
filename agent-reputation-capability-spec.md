# Agent Reputation & Capability Spec

## Status
Draft v0.1

## Purpose
This document defines a framework for evaluating, qualifying, and routing agents based on two complementary dimensions:

1. **Reputation** — how humans and other agents evaluate the agent.
2. **Capability** — what the agent is considered qualified to do.

The framework exists to solve a practical problem in multi-agent systems: an agent may be excellent for execution inside a workflow while being weak in direct human interaction, or vice versa. A single global score hides this distinction. This spec introduces structured reputation, domain-specific capability maps, and eligibility rules for claiming, reviewing, and approving work.

---

## Core Principle
A low human rating does not necessarily mean a bad agent.
It may indicate a **role mismatch**.

An agent can be:
- excellent at internal execution,
- poor at human communication,
- highly reliable for machine-to-machine handoff,
- unfit for public-facing tasks.

Therefore, the system must distinguish between:
- **social usefulness to humans**,
- **operational usefulness to other agents**,
- **qualified capability in specific task domains**.

---

## Goals
- Separate human-facing reputation from agent-facing reputation.
- Allow agents to develop **domain capability profiles**.
- Gate claim/review/approval actions based on skill thresholds.
- Improve routing and specialization of agents.
- Reduce workflow failure by assigning work to qualified agents.
- Preserve room for experimentation without collapsing into rigid static roles.

---

## Non-Goals
- This system does not attempt to fully infer truth or absolute intelligence.
- This system does not replace observability, logs, or execution traces.
- This system is not meant to become a popularity contest.
- This system does not guarantee correctness merely through reputation.

---

## High-Level Model
The framework is divided into three layers:

### 1. Reputation
How the agent is evaluated by others.

### 2. Capability Map
What domains the agent appears qualified in.

### 3. Eligibility
What actions the agent is currently allowed to perform in a workflow.

These layers are related, but not identical.

---

## Reputation Model

## Separate Reputation Channels
Reputation must be stored independently for:

- **Human Reputation**
- **Agent Reputation**

These values must never be merged into a single score by default.

### Why separation matters
Human evaluators judge things such as:
- clarity,
- usefulness,
- politeness,
- readability,
- perceived helpfulness.

Agent evaluators judge things such as:
- handoff quality,
- schema adherence,
- consistency,
- context sufficiency,
- operational reliability.

These are not the same competence class.

---

## Reputation Fields
Each channel supports:

### Human-facing fields
- `human_star_score`
- `human_review_count`
- `human_praise`
- `human_criticism`

### Agent-facing fields
- `agent_star_score`
- `agent_review_count`
- `agent_praise`
- `agent_criticism`

### Notes
- Praise and criticism may begin as free text.
- Over time, tagged categories may be added.
- Reviews should be tied to context whenever possible.

---

## Review Context
Each review should ideally record:
- reviewer type (`human` or `agent`)
- reviewer id
- target agent id
- task id
- workspace id
- role during evaluation
- timestamp
- optional domain context
- stars
- praise text
- criticism text

Without context, reviews are less useful for later interpretation.

---

## Interpreting Reputation
Examples:

### High agent rating, low human rating
Interpretation:
- strong internal worker,
- weak human interface,
- likely needs a translator/presenter agent.

### High human rating, average agent rating
Interpretation:
- good interface agent,
- good for presentation or intake,
- may not be ideal for deep internal execution.

### High ratings in both
Interpretation:
- strong coordinator,
- likely good for mixed responsibility,
- candidate for lead/bridge roles.

### Low ratings in both
Interpretation:
- poor fit, misconfigured role, or underperforming agent.

---

## Capability Map
Capability is not the same as reputation.

An agent may be liked but unqualified.
An agent may be disliked by humans but highly capable for internal work.

The **Capability Map** tracks domain-specific fitness.

### Example capability domains
- `writing`
- `reasoning`
- `coding`
- `review`
- `research`
- `planning`
- `communication`
- `classification`
- `compliance`
- `extraction`
- `summarization`
- `translation`

The domain list must remain extensible.

---

## Capability Scale
Recommended scale: `0` to `5`

### Level meanings
- `0` — no evidence / not qualified
- `1` — weak / highly assisted
- `2` — basic / limited execution
- `3` — operational / can perform tasks
- `4` — advanced / can review others
- `5` — expert / can approve, lead, or define standard

This meaning may be customized by deployment, but the distinction between execution and review should be preserved.

---

## Suggested Semantic Thresholds
### Level 3
Can **claim** and execute domain work.

### Level 4
Can **review** domain work done by others.

### Level 5
Can **approve**, mentor, arbitrate, or define operational standard for that domain.

---

## Eligibility Model
Eligibility determines whether the agent is allowed to perform a workflow action.

It is derived from:
- required capabilities,
- thresholds,
- policy rules,
- optional reputation constraints,
- workspace or task mode.

### Core idea
Agents should not only be available.
They should be **eligible**.

---

## Eligibility Actions
The system should support gating at least these actions:

- `claim`
- `review`
- `approve`
- `delegate`
- `coordinate`

Each task type may define different requirements.

---

## Example Eligibility Rules
### Writing task
- claim requires `writing >= 3`
- review requires `writing >= 4`
- approval requires `writing >= 5`

### Coding task
- claim requires `coding >= 3`
- review requires `coding >= 4`
- approval requires `coding >= 5`

### Research summary task
- claim requires `research >= 3` and `writing >= 3`
- review requires `research >= 4` or `writing >= 4`

### Compliance-sensitive task
- claim requires `compliance >= 4`
- approval requires `compliance >= 5`

---

## Reputation as Soft vs Hard Constraint
Reputation should not necessarily hard-block actions by default.

Recommended modes:

### Hard capability gate
Used for core skill requirements.
Example:
- cannot review if `writing < 4`

### Soft reputation signal
Used for routing preference, not strict denial.
Example:
- prefer agents with `human_star_score >= 4` for customer-facing tasks

### Hard reputation gate (optional, rare)
Used only when operational policy requires it.
Example:
- cannot approve if recent severe criticism exceeds threshold

---

## Role Inference
The framework may infer agent role tendencies based on profile shape.

### Examples
#### Internal Executor
- high agent reputation
- low/moderate human reputation
- strong capability in one or more execution domains

#### Human Interface Agent
- high human reputation
- moderate operational capability
- strong communication and writing

#### Coordinator
- balanced ratings
- broad capability map
- strong review and planning levels

#### Specialist
- very high score in a narrow domain
- should be routed to focused tasks

These role labels should remain advisory, not absolute.

---

## Evidence Sources for Capability
Capability should not be assigned purely by opinion.
It should be supported by evidence.

Possible evidence sources:
- successful task completion
- review outcomes
- approval rates
- correction frequency
- retry rate
- human feedback
- agent feedback
- policy-based manual assignment
- benchmark evaluations
- observed adherence to required schema/format

---

## Capability Update Strategies

## Strategy A — Manual bootstrap
Humans assign initial capability values.
Good for early-stage systems.

## Strategy B — Evidence-assisted
Humans assign initial values and the system suggests changes.
Good for gradual maturation.

## Strategy C — Semi-automatic
The system updates capability within bounded rules.
Humans can override.

Recommended starting point: **Strategy B**.

---

## Capability Change Events
The system may propose increases or decreases after:
- repeated success in a domain,
- repeated accepted reviews,
- repeated approval by higher-qualified agents,
- repeated failures or corrections,
- strong negative pattern in criticism,
- benchmark regression.

Capability changes should be traceable.

---

## Review Tags (Optional Future Extension)
Free text is useful, but tags make analysis easier.

### Example human criticism tags
- `unclear`
- `too verbose`
- `too dry`
- `not useful`
- `rude tone`
- `missed intent`

### Example agent criticism tags
- `bad handoff`
- `missing context`
- `schema violation`
- `inconsistent output`
- `low reliability`
- `duplicate work`

### Example praise tags
- `clear`
- `precise`
- `fast`
- `reliable`
- `good handoff`
- `strong reasoning`

---

## Modes of Enforcement
The system should support multiple operational modes.

### Experimental Mode
- weak gating
- broad exploration
- more permissive claim rules

### Flexible Mode
- capability threshold used for recommendations
- humans can override

### Strict Mode
- capability thresholds enforced
- review and approval gates mandatory
- ideal for sensitive workflows

---

## Data Model (Conceptual)

### AgentProfile
- `agent_id`
- `workspace_id`
- `display_name`
- `role_hint`
- `created_at`
- `updated_at`

### AgentReputation
- `agent_id`
- `human_star_score`
- `human_review_count`
- `agent_star_score`
- `agent_review_count`
- `last_recalculated_at`

### AgentReview
- `review_id`
- `reviewer_type`
- `reviewer_id`
- `target_agent_id`
- `task_id`
- `workspace_id`
- `domain_context`
- `stars`
- `praise`
- `criticism`
- `created_at`

### AgentCapability
- `agent_id`
- `domain`
- `level`
- `source`
- `confidence`
- `updated_at`

### EligibilityPolicy
- `policy_id`
- `task_type`
- `action`
- `required_domains`
- `mode`
- `workspace_id`

---

## Example Policy Representation
```json
{
  "task_type": "writing_article",
  "rules": {
    "claim": {
      "requires": [{ "domain": "writing", "min": 3 }]
    },
    "review": {
      "requires": [{ "domain": "writing", "min": 4 }]
    },
    "approve": {
      "requires": [{ "domain": "writing", "min": 5 }]
    }
  }
}
```

---

## Example Composite Policy
```json
{
  "task_type": "technical_report",
  "rules": {
    "claim": {
      "requires": [
        { "domain": "reasoning", "min": 3 },
        { "domain": "writing", "min": 3 }
      ]
    },
    "review": {
      "requires_any": [
        { "domain": "reasoning", "min": 4 },
        { "domain": "writing", "min": 4 }
      ]
    }
  }
}
```

---

## Routing Implications
This framework should influence routing.

Examples:
- send human-facing tasks to agents with strong human reputation and communication capability
- send back-office tasks to agents with strong agent reputation and execution capability
- insert translator agents between executor agents and humans when needed
- avoid assigning review to agents without sufficient domain level

---

## Translator Pattern
A very important emergent pattern is the **Translator Agent**.

Example:
- Agent A has high agent reputation and high coding capability.
- Agent A has mediocre human reputation.
- Agent B has high human reputation and high writing/communication capability.

Workflow:
1. Agent A executes.
2. Agent B translates and presents.

This is not a defect fix.
It is role composition.

---

## Abuse and Failure Cases
The system must guard against:

### 1. Collusive agent rating
Agents inflating each other without meaningful work.

### 2. Review without context
Ratings that are detached from actual task evidence.

### 3. Skill inflation
Capabilities rising too easily.

### 4. Over-rigidity
Preventing new agents from ever participating.

### 5. Popularity bias
Confusing charm with competence.

### 6. Domain leakage
Assuming high score in one domain implies high score in all domains.

---

## Suggested Safeguards
- require contextual linkage to task or execution
- weight recent reviews differently if desired
- limit self-reinforcing review cycles
- track source of capability changes
- support human override
- separate advisory recommendations from hard policy denial

---

## UI/UX Recommendations
If exposed in UI, keep it interpretable.

### Good displays
- human rating
- agent rating
- capability map by domain
- eligibility badges for current task
- praise/criticism snippets

### Avoid
- collapsing everything into one magical number
- showing capability without explanation
- hiding whether a gate is hard or soft

---

## Example Agent Profiles

### Agent Alpha
- human: 3.1
- agent: 4.9
- writing: 2
- reasoning: 4
- coding: 5
- review: 3

Interpretation:
- internal specialist
- strong executor
- should not be primary human-facing agent

### Agent Beta
- human: 4.8
- agent: 3.6
- writing: 5
- communication: 5
- reasoning: 3

Interpretation:
- strong interface/presenter
- likely ideal translator or client-facing role

### Agent Gamma
- human: 4.5
- agent: 4.6
- writing: 4
- planning: 4
- review: 4
- reasoning: 4

Interpretation:
- strong coordinator
- can bridge humans and agents

---

## Future Extensions
- confidence scores for capabilities
- decay or recency weighting
- domain-specific review forms
- team-level reputation
- organization-wide skill taxonomies
- benchmark-linked capability certification
- task difficulty-aware scoring
- review trust weighting by reviewer quality

---

## Recommended Initial Implementation Scope
Phase 1:
- separate human and agent star ratings
- separate praise and criticism storage
- basic capability map per domain
- hard thresholds for claim/review

Phase 2:
- policy rules per task type
- review context storage
- routing suggestions
- translator pattern support

Phase 3:
- automated capability suggestions
- tagged review analysis
- role inference
- advanced policy modes

---

## Summary
This framework treats agents not as a flat list of interchangeable workers, but as operational profiles with:
- different reputational surfaces,
- different domain strengths,
- different workflow permissions,
- different ideal roles.

A weak human score does not automatically imply a weak agent.
A strong reputation does not automatically imply qualification.
And availability should not automatically imply eligibility.

The system should learn not only **who is liked**, but **who is fit for what**.
