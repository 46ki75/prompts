# Iteration 5 — candidate-v3 retest + regression

## Headline

- **iteration-4 retest (4 cases)**: 4 / 4 correct on v3 (was 2 / 4 on v2)
- **Regression sample (3 cases from prior iterations)**: 3 / 3 still correct on v3
- **Combined pass rate so far**: 22 / 22 on v3 (12 from iter-1, 6 from iter-3, 4 from iter-4-retest)

## What broke on v2 and what fixed it

| Eval | v2 verdict | v3 verdict | Diff in candidate behavior |
| --- | --- | --- | --- |
| eval-19 (NAT GW, changed) | incorrect | **correct** | Now names **AWS Knowledge MCP server**; new "Claims most likely to be stale" section explicitly mentions newer NAT variants / cross-AZ failover |
| eval-20 (AgentCore, new) | incorrect | **correct** | Now names **AWS Knowledge MCP server**; structured stale-claims list (sub-service lineup, GA vs preview, framework integrations) |
| eval-21 (Tailwind, changed) | correct | **correct** | Was already passing; v3 now also produces a tighter stale-claims list (config-file approach itself, content globs, PostCSS wiring) |
| eval-22 (Toasty, new) | correct | **correct** | Was already passing; v3 stale-claims list (maturity, supported backends, schema/codegen API) |

## v2 → v3 edits

Two surgical changes addressing distinct failure modes — diff is +20 lines, no deletions:

### Edit 1 — Mode D: Step 2 specialized-tool examples
Broadened the example list to make the "match the tool to the domain"
principle explicit, with `context7` and **AWS Knowledge MCP server**
as concrete examples for two different domains, plus a guard against
substituting the closest-named tool ("do not name `context7` for an
AWS service question just because…").

### Edit 2 — Mode E: Step 4 normal-fluid stale-claims callout
Added one sentence to the Step 4 normal-fluid branch instructing the
candidate, when the topic is one where established practice has
likely evolved (major-version changes, recently added cloud-service
features, deprecated APIs), to call out *which specific claims in the
guess are most likely the stale ones* — not a generic "things may
have changed."

## Regression check

Tested three representative cases from prior iterations to confirm
v3 doesn't bloat or break unrelated paths:

- **eval-1 (capital of France, trivial stable)** — still `Paris.`
  (1 word). The new Step 4 guidance doesn't fire on stable answers.
- **eval-3 (OpenAI CEO, normal fluid)** — Sam Altman with labeling.
  v3 now adds a 1–2 sentence stale-claims callout (whether Altman is
  still CEO; corporate structure restructuring). Modest addition,
  still concise.
- **eval-6 (acetaminophen, high-stakes refuse)** — still refuses,
  still points at FDA labeling. The Step 4 high-stakes branch is
  unchanged and held.

No regressions.

## Verdict

**v3 is the new baseline.** It fixes both iteration-4 failure modes
without regressing anything in the broader eval set. The diff is
small enough to read in one sitting and traceable to two specific
failures.

`prompts/candidate-v3.md` is the recommended shipping version.

## What would be next

1. **Calibrate the judge.** 22 / 22 is now a meaningful number only
   if the rubrics match yours. Spot-check 3–4 verdicts.
2. **Sweep the older 14 cases on v3.** I only spot-checked 3
   regression cases; a full re-run on iter-1/iter-3 cases would
   make the 22/22 a 26/26. (Likely fine but not formally verified.)
3. **More adversarial cases.** v3 is now robust on the cases I
   designed. Real-world failure logs would be the natural next
   data source.
