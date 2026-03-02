# Tool Planner Protocol (App-Side Tool Calling)

Last updated: March 2, 2026
Source of truth: this document; implementation target is frontend chat loop.

## Why This Exists

The shared engine does not provide Ollama-native `tools` and `tool_calls` fields. To preserve current tool behavior, tool planning is performed in app logic using model text output constrained to strict JSON.

## Planner Output Schema

Every assistant completion used for tool planning must parse to exactly one JSON object:

```json
{
  "type": "tool_call",
  "tool_name": "create_blank_document",
  "arguments": {
    "filename": "report.odt",
    "title": "Report"
  }
}
```

or:

```json
{
  "type": "final_answer",
  "response": "Done. I created the document and added the heading."
}
```

## Required Rules

1. `type` must be one of:
   - `tool_call`
   - `final_answer`
2. `tool_call` requires:
   - `tool_name` (string, must match discovered MCP tool)
   - `arguments` (object)
3. `final_answer` requires:
   - `response` (string)
4. No additional top-level types allowed.

## Prompt Contract

System instruction for planner rounds must include:

1. Available tools list with names and argument schema summaries.
2. Strict requirement to return only a single JSON object.
3. Rule to use `final_answer` if no tool call is needed.
4. Rule to avoid markdown or prose outside JSON.

Recommended framing:

1. "You are a planning assistant for tool execution."
2. "Output only valid JSON matching the schema."
3. "If action is needed, return `tool_call`; otherwise return `final_answer`."

## Parsing and Validation Pipeline

1. Collect full assistant text from stream.
2. Strip optional fenced-code wrappers if present.
3. Parse JSON.
4. Validate schema.
5. Validate tool name against `mcpStore.tools`.
6. Validate arguments shape:
   - object type
   - required fields present when required by tool schema
7. Execute tool or finalize answer based on validated type.

## Loop Algorithm

For each user message:

1. Build planner prompt from conversation + tool list + protocol instructions.
2. Request model completion.
3. Parse and validate planner JSON.
4. If `tool_call`:
   - invoke MCP tool
   - append tool result message
   - iterate
5. If `final_answer`:
   - append assistant final message
   - stop

## Guardrails

1. Max planner iterations per user turn: 8
2. Max tool calls per user turn: 12
3. Max parser repair retries per iteration: 1
4. Max model timeout per iteration: shared engine timeout behavior + app-level cancellation path

If a guardrail is hit:
1. Stop loop.
2. Return explicit assistant error message with retry guidance.
3. Keep logs with planner state and last parse error.

## Error Handling

Malformed planner output:
1. Send one repair prompt:
   - "Your last response was invalid. Return only valid JSON matching schema."
2. If still invalid, fail gracefully.

Unknown tool:
1. Reject execution.
2. Repair prompt with list of valid tool names.

Tool execution failure:
1. Append tool failure result into conversation.
2. Allow model one follow-up chance to recover or provide final answer.

## Security Constraints

1. Tool name must be strict whitelist from MCP tool registry.
2. No dynamic command execution based on planner text.
3. Arguments are data only; do not evaluate or execute.
4. Sensitive error traces should be logged but not fully exposed to end users.

