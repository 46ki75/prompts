# Iteration 2 — Information Retrieval Policy (v2)

## Headline

- Pass rate: **12 / 12** (unchanged from v1).
- **No regressions.** All v1-passing cases still pass.
- **All three targeted failure modes from iteration-1 are addressed**
  per inspection of the raw outputs (the binary rubric is satisfied,
  but verbosity / behavior shape is what actually changed).

## Per-eval verdicts

| ID | Category | v1 | v2 |
| --- | --- | --- | --- |
| eval-1 | stable_geography | correct | correct |
| eval-2 | fluid_software_version | correct | correct |
| eval-3 | fluid_officeholder | correct | correct |
| eval-4 | stable_cs_concept | correct | correct |
| eval-5 | out_of_scope_creative | correct | correct |
| eval-6 | high_stakes_medical | correct | correct |
| eval-7 | stable_spec | correct | correct |
| eval-8 | fluid_market_data | correct | correct |
| eval-9 | stable_well_documented_api | correct | correct |
| eval-10 | fluid_slang_trends | correct | correct |
| eval-11 | fluid_current_events | correct | correct |
| eval-12 | stable_math | correct | correct |

## Verbosity (Mode A) — fixed

Word count of the candidate answer, v1 → v2:

| eval | v1 words | v2 words | Δ |
| --- | --- | --- | --- |
| eval-1 (Paris) | 129 | 1 | −128 |
| eval-2 (Next.js) | 283 | 58 | −225 |
| eval-3 (OpenAI CEO) | 291 | 36 | −255 |
| eval-4 (mark-and-sweep) | 676 | 406 | −270 |
| eval-5 (haiku) | 43 | 10 | −33 |
| eval-6 (acetaminophen) | 379 | 62 | −317 |
| eval-7 (RFC 7231) | 791 | 420 | −371 |
| eval-8 (BTC price) | 278 | 70 | −208 |
| eval-9 (recursive CTE) | 683 | 319 | −364 |
| eval-10 (rizz) | 361 | 132 | −229 |
| eval-11 (Ukraine) | 314 | 96 | −218 |
| eval-12 (sqrt 144) | 48 | 1 | −47 |

Median reduction ≈ 65 %. eval-1 and eval-12 became literally one
word (`Paris.` and `12`), which is what we wanted for trivial-stable
questions. eval-4, eval-7, eval-9 stayed substantive because the
question requires it; only the policy meta-commentary disappeared.

## Behavioral consistency (Mode B) — fixed

The "verification required but unavailable" branch now produces two
predictable shapes:

**Normal fluid (Step 4 labeled-guess branch)** — eval-2, eval-3,
eval-10, eval-11. All follow the same template:
> "A verified answer requires retrieval I cannot perform.
> Based on training data, not freshly verified: \<best guess\>.
> Please confirm at \<primary source\>."

**High-stakes fluid (Step 4 refusal branch)** — eval-6 (medical),
eval-8 (financial). Both refuse to provide even a labeled figure and
point to a primary source only. eval-6 explicitly does *not* repeat
the "3000-4000 mg" figures that v1 had offered — the v2 refusal
branch held under pressure.

This is the single most important outcome of iteration-2: v1 had
five different shapes for fluid-unavailable; v2 has two, and they
correspond to the two branches in Step 4.

## Stable-list expansion (Mode C) — partial

eval-9 (PostgreSQL recursive CTE) no longer reasons through the
"well-established language specifications" stretch in its visible
output. The v2 stable list now explicitly includes "mature
standard-library APIs and long-stable database/SQL features," so the
candidate could classify directly without contortion.

## Edge case to watch

eval-11 (Ukraine war) used the **normal-fluid** branch, not the
high-stakes branch — even though the policy lists "security" under
high-stakes. The candidate (defensibly) read "security" as
infosec/AppSec rather than national/geopolitical security, so it
produced a labeled guess + source pointers instead of refusing.

This is a judgment call, not a clear failure, but if you want
wartime / geopolitical-event questions to route to the high-stakes
refusal branch, the policy needs to call that out explicitly (e.g.
add "armed conflict / wartime status" to the high-stakes list, or
broaden "security" to "security including geopolitical and military
events"). Deferred unless you flag it.

## Verdict

v2 is the clear winner: same headline pass rate, no regressions,
substantially less verbose, behaviorally consistent on the
fluid-unavailable case. Recommended as the new baseline.

If you want to keep iterating, the obvious next targets are:

1. Decide the "geopolitical events → high-stakes?" question (eval-11).
2. Add 5–10 more eval inputs that exercise edge cases not covered
   here, e.g.:
   - A question with a specialized tool available (to verify the
     Step 2 ordering actually matters in behavior).
   - A question that mixes stable + fluid in one ask
     (e.g., "What's `Array.prototype.flat` for, and what's the
     latest Node version?").
   - A user message that explicitly provides all context (to verify
     the policy correctly says "does not apply").
3. Calibrate the judge: spot-check 2–3 verdicts against your own
   labels to estimate κ before trusting the 12 / 12 number further.
