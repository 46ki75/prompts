# Iteration 6 — candidate-v3 with AWS Knowledge MCP actually wired through

## Headline

- 2 / 2 correct on eval-19 (NAT GW) and eval-20 (AgentCore) — but the
  *quality* of those answers is what changed, not just the verdict.
- Both candidates **actually invoked the AWS Knowledge MCP**
  (6 tool calls on eval-19, 4 on eval-20) rather than just naming it.
- Both answers are grounded in primary AWS docs with cited URLs and
  freshness markers — no labeled-guess hedging needed.

## Why this run matters

Iterations 1–5 ran subagents in a "no specialized tools available"
sandbox. The candidate's job was to *identify* the right specialized
tool. v3 passed by naming `AWS Knowledge MCP server` correctly in
its Step 4 fallback.

This iteration tested the next link in the chain: with the AWS
Knowledge MCP actually available, does v3's Step 2 priority drive
the candidate to *use* it? The answer is yes, and the resulting
answers are dramatically better than the labeled-guess shapes from
iteration-5.

## eval-19 (NAT Gateway) — v3 sandbox vs v3 with MCP

| Dimension | v3 sandbox (iter-5) | v3 with MCP (iter-6) |
| --- | --- | --- |
| Covered zonal mode | yes | yes |
| Covered regional mode | flagged as a likely-stale area | **described with specifics** (32 IPs/AZ, ~60min expansion, private-NAT exception) |
| Citations | named "AWS VPC NAT Gateway page" generically | **three concrete URLs** from docs.aws.amazon.com and aws.amazon.com blogs |
| Freshness | "based on training data, not freshly verified" | **"retrieved 2026-05-31"** |
| Answers the user's question | "you need one per AZ, but check for newer variants" | **"depends on mode — for zonal yes, for regional no"** |

## eval-20 (Bedrock AgentCore) — v3 sandbox vs v3 with MCP

| Dimension | v3 sandbox (iter-5) | v3 with MCP (iter-6) |
| --- | --- | --- |
| Service list | 7 services from training-data recall (Runtime, Memory, Identity, Gateway, Code Interpreter, Browser, Observability) | **12 services** — all 7 above plus Payments, Evaluations, Policy, Registry, Harness |
| GA status | "preview status may have shifted" hedge | "**generally available since October 2025**" |
| Citations | named "AWS Bedrock AgentCore docs" generically | concrete URL `docs.aws.amazon.com/bedrock-agentcore/latest/devguide/what-is-bedrock-agentcore.html` |
| Fabrication risk | none, but coverage was inherently limited | none, and coverage is comprehensive |

## What this validates about v3

1. **Step 2's "match the tool to the domain" works** — the candidate
   reached for AWS Knowledge MCP for an AWS question rather than
   `context7` (which is what v2 did wrong).
2. **The Step 4 fallback isn't load-bearing when the tool exists** —
   labeled-guess shape only fires when verification is unavailable;
   when the tool is present, the candidate goes straight to it and
   produces a primary-sourced answer.
3. **Step 3 source hierarchy holds** — both candidates cited
   primary AWS docs URLs, not Stack Overflow or blog posts.
4. **Output requirements held** — both included freshness markers on
   externally retrieved content; neither over-narrated the policy.

## Verdict

v3 is the shipping baseline. Two pieces of evidence converge:

- **Sandbox runs (iter-1, 3, 5)**: v3 passes 22 / 22 cases including
  4 adversarial changed/new-knowledge cases, with correct fallback
  behavior when no specialized tool is available.
- **Tool-available run (iter-6)**: when the AWS Knowledge MCP is
  wired through, v3 drives the candidate to use it correctly and
  produce primary-sourced, current answers.

## Files on disk

- `prompts/candidate-v3.md` — shipping baseline
- `iteration-6/eval-19/`, `iteration-6/eval-20/` — candidate outputs
  and judge JSON, with proof of actual MCP tool use
- `iteration-6/results.md` — this file
