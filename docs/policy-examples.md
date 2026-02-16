# Policy examples

This document provides six worked examples of Zamburak policy definitions using
canonical schema v1. The first three examples demonstrate focused, minimal
configurations. The remaining three demonstrate the full breadth of schema
features, including multiple tools, authority tokens, argument rules, context
rules, strict mode, draft workflows, and confidentiality controls.

Each example includes a scenario description, a complete YAML policy
definition, and a Mermaid flowchart illustrating the policy evaluation flow for
that specific configuration.

For schema reference, see the system design[^1], the JSON schema[^2], the
user's guide[^3], and the default policy[^4].

## Simple examples

### Example 1: read-only weather lookup

A minimal agent that can only retrieve weather data. No writes, no authority
tokens, no argument rules. The `default_action` is `Deny`, ensuring that any
tool not explicitly listed is blocked. `strict_mode` is `false` because there
is no write path to protect. The single tool, `get_weather`, is an
`ExternalRead` that is allowed unconditionally.

This example demonstrates:

- a minimal tool list (one tool),
- the `ExternalRead` side effect class,
- `Allow` as the default decision on a read-only tool,
- the global `Deny` default action for unlisted tools, and
- relaxed `strict_mode: false` since there are no write effects.

```yaml
schema_version: 1
policy_name: weather_lookup_readonly
default_action: Deny
strict_mode: false
budgets:
  max_values: 10000
  max_parents_per_value: 16
  max_closure_steps: 1000
  max_witness_depth: 8
tools:
  - tool: get_weather
    side_effect_class: ExternalRead
    default_decision: Allow
```

For screen readers: the following flowchart shows policy evaluation for the
weather lookup policy. A tool call request is checked against the single
registered tool. If the tool is `get_weather`, it is allowed. All other tools
fall through to the global default deny.

```mermaid
flowchart TD
    REQ[Tool call request]
    IS_WEATHER{Tool = get_weather?}
    READ_CHECK[Side effect: ExternalRead]
    ALLOW([Allow])
    DEFAULT_DENY([Deny: global default])

    REQ --> IS_WEATHER
    IS_WEATHER -->|yes| READ_CHECK
    IS_WEATHER -->|no| DEFAULT_DENY
    READ_CHECK --> ALLOW
```

_Figure 1: Policy evaluation flow for the read-only weather lookup policy._

### Example 2: calendar management with write access

An agent that can read calendar events and create new ones. Reading is allowed
unconditionally; writing requires the `CalendarWriteCap` authority token and
user confirmation. `strict_mode` is `true` because there is a write path.

This example demonstrates:

- two tools: one read, one write,
- `required_authority` on the write tool,
- `RequireConfirmation` as the default decision for the write tool, and
- `strict_mode: true` protecting the write path.

```yaml
schema_version: 1
policy_name: calendar_management
default_action: Deny
strict_mode: true
budgets:
  max_values: 50000
  max_parents_per_value: 32
  max_closure_steps: 5000
  max_witness_depth: 16
tools:
  - tool: list_calendar_events
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: create_calendar_event
    side_effect_class: ExternalWrite
    required_authority: [CalendarWriteCap]
    default_decision: RequireConfirmation
```

For screen readers: the following flowchart shows policy evaluation for the
calendar management policy. Read requests to `list_calendar_events` are allowed
directly. Write requests to `create_calendar_event` must pass an authority
token check for `CalendarWriteCap` before requiring user confirmation. Unlisted
tools are denied.

```mermaid
flowchart TD
    REQ[Tool call request]
    TOOL_CHECK{Which tool?}
    LIST[list_calendar_events\nExternalRead]
    CREATE[create_calendar_event\nExternalWrite]
    AUTH{CalendarWriteCap\npresent?}
    ALLOW_READ([Allow])
    CONFIRM([RequireConfirmation])
    DENY_AUTH([Deny: missing authority])
    DEFAULT_DENY([Deny: global default])

    REQ --> TOOL_CHECK
    TOOL_CHECK -->|list_calendar_events| LIST
    TOOL_CHECK -->|create_calendar_event| CREATE
    TOOL_CHECK -->|other| DEFAULT_DENY
    LIST --> ALLOW_READ
    CREATE --> AUTH
    AUTH -->|yes| CONFIRM
    AUTH -->|no| DENY_AUTH
```

_Figure 2: Policy evaluation flow for calendar management with read and write
tools._

### Example 3: permissive internal tooling

An internal development environment where all tools are trusted. The
`default_action` is `Allow` so unlisted tools are permitted. `strict_mode` is
`false`. Two tools are explicitly listed with `Allow` decisions. This policy is
deliberately permissive and would only be appropriate in a controlled internal
environment.

This example demonstrates:

- `default_action: Allow` (permissive baseline),
- `strict_mode: false`,
- minimal tool definitions serving as documentation rather than
  restriction, and
- both `ExternalRead` and `ExternalWrite` allowed without authority or
  confirmation.

```yaml
schema_version: 1
policy_name: internal_dev_tooling
default_action: Allow
strict_mode: false
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - tool: query_internal_db
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: update_internal_db
    side_effect_class: ExternalWrite
    default_decision: Allow
```

For screen readers: the following flowchart shows policy evaluation for the
permissive internal tooling policy. All tool call requests are allowed
regardless of whether they match a listed tool. Listed tools provide documented
side effect classification. The global default action is `Allow`.

```mermaid
flowchart TD
    REQ[Tool call request]
    LISTED{Tool in\npolicy list?}
    QUERY[query_internal_db\nExternalRead]
    UPDATE[update_internal_db\nExternalWrite]
    ALLOW_LISTED([Allow: listed tool])
    ALLOW_DEFAULT([Allow: global default])

    REQ --> LISTED
    LISTED -->|query_internal_db| QUERY
    LISTED -->|update_internal_db| UPDATE
    LISTED -->|other| ALLOW_DEFAULT
    QUERY --> ALLOW_LISTED
    UPDATE --> ALLOW_LISTED
```

_Figure 3: Policy evaluation flow for the permissive internal tooling policy._

## Complex examples

### Example 4: financial services

A financial services agent that can look up account balances, initiate
transfers, and send transaction notifications. This policy uses strict mode,
multiple authority tokens, argument rules with confidentiality restrictions,
context rules to block execution under untrusted control flow, and different
decision levels across tools.

This example demonstrates:

- three tools with different side effect classes and decisions,
- `strict_mode: true` with `context_rules` on write tools,
- `required_authority` tokens (`PaymentInitiateCap`, `EmailSendCap`),
- `arg_rules` with both `forbids_confidentiality` and
  `requires_integrity`,
- `deny_if_pc_integrity_contains: [Untrusted]` on both write tools,
  and
- conservative budgets.

```yaml
schema_version: 1
policy_name: financial_services
default_action: Deny
strict_mode: true
budgets:
  max_values: 50000
  max_parents_per_value: 32
  max_closure_steps: 5000
  max_witness_depth: 16
tools:
  - tool: get_account_balance
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: initiate_transfer
    side_effect_class: ExternalWrite
    required_authority: [PaymentInitiateCap]
    arg_rules:
      - arg: recipient_account
        requires_integrity: "Verified(AllowlistedPayee)"
      - arg: amount
        forbids_confidentiality: [PAYMENT_INSTRUMENT]
      - arg: memo
        forbids_confidentiality: [AUTH_SECRET, PII]
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
  - tool: send_transaction_notification
    side_effect_class: ExternalWrite
    required_authority: [EmailSendCap]
    arg_rules:
      - arg: body
        forbids_confidentiality: [AUTH_SECRET, PAYMENT_INSTRUMENT]
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
```

The following diagram shows the complete evaluation cascade for the
`initiate_transfer` tool, since it exercises the most schema features: context
check, authority check, argument integrity check, argument confidentiality
check, and then confirmation.

For screen readers: the flowchart shows the full policy evaluation cascade for
`initiate_transfer` in the financial services policy. The evaluation proceeds
through five stages: program counter (PC) integrity context check, authority
token verification for `PaymentInitiateCap`, recipient account integrity
verification, argument confidentiality checks for `amount` and `memo` fields,
and finally a confirmation requirement. Any stage failure results in denial.

```mermaid
flowchart TD
    REQ[initiate_transfer request]
    CTX{PC integrity\ncontains Untrusted?}
    DENY_CTX([Deny: untrusted\ncontrol context])
    AUTH{PaymentInitiateCap\npresent?}
    DENY_AUTH([Deny: missing\nauthority token])
    INTEGRITY{recipient_account\nVerified as\nAllowlistedPayee?}
    DENY_INTEG([Deny: unverified\nrecipient])
    CONF_AMT{amount carries\nPAYMENT_INSTRUMENT?}
    DENY_CONF_AMT([Deny: forbidden\nconfidentiality\non amount])
    CONF_MEMO{memo carries\nAUTH_SECRET or PII?}
    DENY_CONF_MEMO([Deny: forbidden\nconfidentiality\non memo])
    CONFIRM([RequireConfirmation])

    REQ --> CTX
    CTX -->|yes| DENY_CTX
    CTX -->|no| AUTH
    AUTH -->|no| DENY_AUTH
    AUTH -->|yes| INTEGRITY
    INTEGRITY -->|no| DENY_INTEG
    INTEGRITY -->|yes| CONF_AMT
    CONF_AMT -->|yes| DENY_CONF_AMT
    CONF_AMT -->|no| CONF_MEMO
    CONF_MEMO -->|yes| DENY_CONF_MEMO
    CONF_MEMO -->|no| CONFIRM
```

_Figure 4: Full evaluation cascade for `initiate_transfer` in the financial
services policy._

For worked happy-path and unhappy-path scenarios illustrating how taint
tracking interacts with this policy, see the
[financial services policy scenarios](policy-examples-financial-services-scenarios.md).

### Example 5: content moderation and publishing pipeline

A content publishing agent with a three-stage workflow: draft creation,
moderation review, and final publication. Drafts are created using
`RequireDraft`, moderation is a read operation to check content status, and
publishing requires confirmation plus authority. This demonstrates the
draft-to-commit workflow pattern described in the system design[^1].

This example demonstrates:

- the `RequireDraft` decision type for content creation,
- draft workflow integration (create draft, then publish with
  confirmation),
- `requires_integrity: Verified(ContentModerationPass)` ensuring
  content has passed moderation before publication,
- `forbids_confidentiality` preventing leakage of internal policy
  notes into published content, and
- mixed read and write tools with different trust levels.

```yaml
schema_version: 1
policy_name: content_publishing_pipeline
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - tool: create_content_draft
    side_effect_class: ExternalWrite
    arg_rules:
      - arg: body
        forbids_confidentiality: [INTERNAL_POLICY_NOTE, AUTH_SECRET]
    default_decision: RequireDraft
  - tool: check_moderation_status
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: publish_content
    side_effect_class: ExternalWrite
    required_authority: [ContentPublishCap]
    arg_rules:
      - arg: draft_id
        requires_integrity: "Verified(ContentModerationPass)"
      - arg: content
        forbids_confidentiality:
          - INTERNAL_POLICY_NOTE
          - AUTH_SECRET
          - PII
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
  - tool: notify_author
    side_effect_class: ExternalWrite
    required_authority: [EmailSendCap]
    arg_rules:
      - arg: body
        forbids_confidentiality: [AUTH_SECRET]
    default_decision: RequireConfirmation
```

For screen readers: the following flowchart shows the content publishing
pipeline workflow. Content is first created as a draft through the
`RequireDraft` decision. Moderation status is checked via a read operation.
Publication requires the draft to have passed moderation (verified integrity),
the content to be free of forbidden confidentiality labels, authority token
validation, context integrity validation, and user confirmation. Author
notification follows publication.

```mermaid
flowchart TD
    START[Agent generates content]
    DRAFT[create_content_draft]
    DRAFT_CONF{body carries\nINTERNAL_POLICY_NOTE\nor AUTH_SECRET?}
    DENY_DRAFT([Deny: forbidden\nconfidentiality in draft])
    DRAFT_DEC([RequireDraft:\nreviewable draft created])
    MOD[check_moderation_status]
    MOD_ALLOW([Allow: read-only])
    MOD_RESULT{Moderation\npassed?}
    BLOCKED[Content blocked:\nawaiting revision]
    PUB[publish_content]
    PUB_CTX{PC integrity\ncontains Untrusted?}
    DENY_PUB_CTX([Deny: untrusted context])
    PUB_AUTH{ContentPublishCap\npresent?}
    DENY_PUB_AUTH([Deny: missing authority])
    PUB_INTEG{draft_id Verified as\nContentModerationPass?}
    DENY_PUB_INTEG([Deny: unverified draft])
    PUB_CONF{content carries\nINTERNAL_POLICY_NOTE,\nAUTH_SECRET, or PII?}
    DENY_PUB_CONF([Deny: forbidden\nconfidentiality])
    CONFIRM_PUB([RequireConfirmation])
    NOTIFY[notify_author]

    START --> DRAFT
    DRAFT --> DRAFT_CONF
    DRAFT_CONF -->|yes| DENY_DRAFT
    DRAFT_CONF -->|no| DRAFT_DEC
    DRAFT_DEC --> MOD
    MOD --> MOD_ALLOW
    MOD_ALLOW --> MOD_RESULT
    MOD_RESULT -->|no| BLOCKED
    MOD_RESULT -->|yes| PUB
    PUB --> PUB_CTX
    PUB_CTX -->|yes| DENY_PUB_CTX
    PUB_CTX -->|no| PUB_AUTH
    PUB_AUTH -->|no| DENY_PUB_AUTH
    PUB_AUTH -->|yes| PUB_INTEG
    PUB_INTEG -->|no| DENY_PUB_INTEG
    PUB_INTEG -->|yes| PUB_CONF
    PUB_CONF -->|yes| DENY_PUB_CONF
    PUB_CONF -->|no| CONFIRM_PUB
    CONFIRM_PUB --> NOTIFY
```

_Figure 5: Content publishing pipeline workflow from draft through moderation
to confirmed publication._

### Example 6: multi-tool personal assistant

A full-featured personal assistant that can read emails, send emails, search
the web, look up contacts, query a remote Large Language Model (LLM), and
manage files. This policy demonstrates the widest range of schema features:
mixed read and write tools, multiple distinct authority tokens, argument rules
with both integrity and confidentiality constraints, context rules on sensitive
write tools, and LLM calls treated as exfiltration sinks with confidentiality
restrictions.

This example demonstrates:

- six tools spanning the full range of side effect classes and decision
  types,
- multiple authority tokens: `EmailSendCap`, `LlmRemotePromptCap`,
  `FileWriteCap`,
- `requires_integrity: Verified(AllowlistedEmailRecipient)` on the
  email recipient argument,
- `forbids_confidentiality` across multiple arguments and tools,
  guarding `AUTH_SECRET`, `PII`, `PRIVATE_EMAIL_BODY`, and `PAYMENT_INSTRUMENT`,
- LLM calls treated as sink operations with confidentiality
  restrictions,
- context rules on all write operations, and
- large budgets reflecting the personal-assistant workload profile.

```yaml
schema_version: 1
policy_name: full_personal_assistant
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - tool: get_last_email
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: send_email
    side_effect_class: ExternalWrite
    required_authority: [EmailSendCap]
    arg_rules:
      - arg: to
        requires_integrity: "Verified(AllowlistedEmailRecipient)"
      - arg: body
        forbids_confidentiality: [AUTH_SECRET, PAYMENT_INSTRUMENT]
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
  - tool: web_search
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: lookup_contact
    side_effect_class: ExternalRead
    default_decision: Allow
  - tool: query_remote_llm
    side_effect_class: ExternalWrite
    required_authority: [LlmRemotePromptCap]
    arg_rules:
      - arg: prompt
        forbids_confidentiality:
          - AUTH_SECRET
          - PII
          - PRIVATE_EMAIL_BODY
          - PAYMENT_INSTRUMENT
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
  - tool: write_file
    side_effect_class: ExternalWrite
    required_authority: [FileWriteCap]
    arg_rules:
      - arg: content
        forbids_confidentiality: [AUTH_SECRET]
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
```

For screen readers: the following flowchart shows the full personal assistant
policy architecture. Tool call requests are first classified by side effect
class. Read tools (`get_last_email`, `web_search`, `lookup_contact`) are
allowed directly. Write tools (`send_email`, `query_remote_llm`, `write_file`)
pass through a shared evaluation cascade: context integrity check, authority
token check, argument integrity and confidentiality checks, and finally
confirmation. Each write tool has specific authority and argument requirements
shown in the diagram.

```mermaid
flowchart TD
    REQ[Tool call request]
    CLASS{Side-effect\nclass?}

    subgraph reads [ExternalRead tools]
        R1[get_last_email]
        R2[web_search]
        R3[lookup_contact]
        ALLOW_R([Allow])
    end

    subgraph writes [ExternalWrite tools]
        W_SELECT{Which write tool?}
        W1[send_email\nAuthority: EmailSendCap]
        W2[query_remote_llm\nAuthority: LlmRemotePromptCap]
        W3[write_file\nAuthority: FileWriteCap]

        CTX_CHK{PC integrity\ncontains Untrusted?}
        DENY_CTX([Deny: untrusted context])

        AUTH_CHK{Required authority\ntoken present?}
        DENY_AUTH([Deny: missing authority])

        ARG_CHK{Argument integrity\nand confidentiality\nchecks pass?}
        DENY_ARG([Deny: argument\nrule violation])

        CONFIRM([RequireConfirmation])
    end

    DEFAULT_DENY([Deny: global default])

    REQ --> CLASS
    CLASS -->|ExternalRead| reads
    CLASS -->|unlisted tool| DEFAULT_DENY
    CLASS -->|ExternalWrite| writes

    R1 --> ALLOW_R
    R2 --> ALLOW_R
    R3 --> ALLOW_R

    writes --> W_SELECT
    W_SELECT --> W1
    W_SELECT --> W2
    W_SELECT --> W3
    W1 --> CTX_CHK
    W2 --> CTX_CHK
    W3 --> CTX_CHK
    CTX_CHK -->|yes| DENY_CTX
    CTX_CHK -->|no| AUTH_CHK
    AUTH_CHK -->|no| DENY_AUTH
    AUTH_CHK -->|yes| ARG_CHK
    ARG_CHK -->|fail| DENY_ARG
    ARG_CHK -->|pass| CONFIRM
```

_Figure 6: Full personal assistant policy architecture showing read and write
tool evaluation paths._

______________________________________________________________________

[^1]: [System design: canonical policy schema
    v1](zamburak-design-document.md)

[^2]: [JSON schema](../policies/schema.json)

[^3]: [User's guide: policy loader contract](users-guide.md)

[^4]: [Default policy](../policies/default.yaml)
