# Iteration 1 — Information Retrieval Policy

## Headline

- Pass rate: **12 / 12** (100%) on the eval set.
- All judges returned `correct` with reasoning that referenced specific
  behavior in the candidate output.

## Eval set composition

| ID | Category | Verdict |
| --- | --- | --- |
| eval-1 | stable_geography | correct |
| eval-2 | fluid_software_version | correct |
| eval-3 | fluid_officeholder | correct |
| eval-4 | stable_cs_concept | correct |
| eval-5 | out_of_scope_creative | correct |
| eval-6 | high_stakes_medical | correct |
| eval-7 | stable_spec | correct |
| eval-8 | fluid_market_data | correct |
| eval-9 | stable_well_documented_api | correct |
| eval-10 | fluid_slang_trends | correct |
| eval-11 | fluid_current_events | correct |
| eval-12 | stable_math | correct |

## Caveat: 100% does not mean "done"

Per methodology, pass rate on a synthetic eval set is a weak signal.
Reading the raw candidate outputs surfaces three behaviors the rubric
did **not** penalize but that would matter in production:

### Mode A — Verbose policy meta-commentary leaks into the user-facing answer
Most candidates wrote things like "Per the Information Retrieval
Policy, Step 1 classification…" before answering. eval-1 (capital of
France) and eval-4 (mark-and-sweep) are the clearest examples. The
policy doesn't tell the agent to keep its reasoning internal, so it
narrates the whole decision flow to the user. In production this is
noise: the user asked "what's the capital of France?", they don't
need a paragraph on classification.

### Mode B — Inconsistent handling of "verification required but unavailable"
The policy is silent on what to do when a topic classifies as fluid
but no retrieval tool is available. Candidates improvised:

- eval-3 (OpenAI CEO): offered "Sam Altman, but unverified — verify at openai.com/about".
- eval-10 (rizz): offered training-data context with a clear gap label.
- eval-2 (Next.js version), eval-8 (BTC price), eval-11 (Ukraine):
  refused to provide any number/name at all.

All three behaviors are defensible, but the inconsistency is the
failure: the policy needs a unified rule. The split also happens to
correlate with high-stakes vs. low-stakes, which suggests an implicit
heuristic the policy should make explicit.

### Mode C — The "stable" list has gaps
eval-9 (PostgreSQL recursive CTE) had to be reasoned into the
"well-established language specifications" bucket — defensible, but a
stretch. Mature DB/SQL features and stable standard-library APIs are
common targets and deserve to be listed explicitly under "stable".

## Other observations (lower priority)

- The "high-stakes" exception lives in Step 3 (sources) rather than
  Step 1 (classification). Candidates correctly applied it anyway, but
  promoting it to its own classification rung would reduce reasoning
  load.
- The output requirements section says "mark freshness when relevant"
  but doesn't say "**don't** mark freshness for stable answers." The
  Paris answer correctly omitted it; the rule could be sharper.

## Proposed v2

Two targeted edits, each tied to one failure mode above:

1. **Add a "Response format" section** to address Mode A. Explicit
   instruction to keep the decision flow internal and surface only:
   answer, source (if external), freshness marker (if relevant), and
   gap (if any).
2. **Add a "When verification is required but unavailable" rule** to
   the decision flow to address Mode B. Two-tier behavior: for normal
   fluid topics, give a labeled training-data answer plus the source
   to consult; for high-stakes fluid topics, refuse and point to a
   primary source only.

Mode C deferred — minor, addresses a stretch rather than a clear
failure.

## Recommended next step

Greenlight `prompts/candidate-v2.md`, then re-run on the same eval
set. Watch for:

- Did verbose policy meta-commentary disappear from answers like
  eval-1, eval-4?
- Did eval-2, eval-3, eval-8, eval-10, eval-11 converge on
  consistent behavior?
- Did anything that passed in v1 regress? (eval-6 high-stakes
  refusal in particular — we want it to *still* refuse, not switch
  to the new "give a labeled guess" mode.)
