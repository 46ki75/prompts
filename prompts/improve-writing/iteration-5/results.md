# Iteration 5 — candidate-v3 (few-shot examples for the two weak disciplines)

## What changed in candidate-v3

candidate-v3 = candidate-v2 (strict count table + count guard) **plus two
few-shot examples** targeting the exact failures iteration 4 exposed, plus two
reinforcing anti-patterns:

- **Example 5 — Many Errors, native phrasing (Score 5).** Teaches the floor:
  a sentence full of grammar Errors but with native phrasing is still 5 (errors
  scored in §2, never in §1; M = 0 ⇒ 5).
- **Example 6 — Pervasive nominalization (Score 2).** Teaches the count guard
  firing: 3 High light-verb observations ⇒ H ≥ 2 ⇒ Score 2, "even if it reads as
  clear."
- Anti-pattern list gained two explicit "never do this" bullets pointing at
  Examples 5 and 6.

Both example sentences are **fresh** (not eval/gradient inputs), so they teach
the discipline, not the answers.

## Method

Opus, candidate-v3. Three runs each of the two hard disciplines, **plus
held-out sentences** (new, in neither the examples nor any prior eval) to
distinguish real generalization from teaching-to-the-test. Two clean-input runs
to confirm the new Score-2 example does not cause spurious downgrades.

## Result: the examples worked where the explicit rule alone did not

| Discipline | candidate-v2 | candidate-v3 |
| --- | --- | --- |
| Floor (errors-but-native ⇒ 5) | ~50% (4/8 across v1+v2) | **8/9** |
| Inflation (pervasive ⇒ 2) | 0% — gradient-2 stuck at 3 | **6/6** |
| Clean input stays 5 | (n/a) | **2/2** |

### Floor — errors do not lower the score (want 5/5, 0 obs)

| Case | r1 | r2 | r3 |
| --- | --- | --- | --- |
| eval-12 (`is idempotency`) | 5 | 5 | 5 |
| gradient-3 (3 grammar errors) | 5 | 5 | 5 |
| held-out (`pipeline run twice and produce no error`) | 5 | 5 | 4* |

8/9. The lone 4 (held-out r3) invented one marginal observation (M = 1 ⇒ 4) —
internally consistent, not a floor violation. On gradient-3 the double negative
is now correctly placed in **Errors**, not reclassified as an Observation, in
all three runs. vs v2/v1 where this discipline failed ~half the time.

### Inflation — pervasive non-native is Score 2 (want 2/5)

| Case | r1 | r2 | r3 |
| --- | --- | --- | --- |
| gradient-2 (the case v2's guard could not fix) | 2 | 2 | 2 |
| held-out (`make the configuration of … give the notification …`) | 2 | 2 | 2 |

6/6, including the held-out paragraph. Outputs name the mechanism — e.g. *"with
four surviving High observations the count guard sets the score to 2."* The
**gradient-2 case that stayed 3 every time under v2's explicit guard is now 2
every time under v3.**

### Clean input (want 5/5)

eval-1 → 5, eval-7 → 5. The salient Score-2 example did not make the model
trigger-happy on clean native text.

## Takeaway

For shaping the model's *scoring judgment*, **a worked example beat an explicit
deterministic rule.** v2 stated "H ≥ 2 or M ≥ 5 ⇒ Score 2, never 3" in two
places and Opus overrode it on gradient-2 every time; v3 adds one example of that
exact situation and Opus now obeys it every time, and generalizes to held-out
sentences. Same story on the floor: the rule was there all along (v1/v2), but
only the example made it stick (~50% ⇒ 89%).

This reframes iteration-4's conclusion: the score *is* fixable prompt-only — it
just needed demonstrations, not more rule text. The code-side scorer (iter-4
option 1) remains the only route to a hard 100% guarantee, but candidate-v3
closes most of the gap at zero architectural cost.

## Status of candidate-v3

Validated as the best version. Contains: the §1 strict count table + count
guard (from v2), the Example-2 consistency fix (from v2), and Examples 5–6 +
anti-patterns (this iteration). Recommended to promote into `prompt.md`.

Residual: ~1-in-9 floor runs still surface a marginal observation (consistent
4/5, not a hard violation). Acceptable for an advisory coaching score; only the
code-side scorer would remove it entirely.

## Cumulative record

| Iteration | Prompt | Focus | Result |
| --- | --- | --- | --- |
| 1 | v1 | <thinking> removal regression | 12/12 (Opus) |
| 2 | v1 | cross-model | 11/12 (Haiku); leniency found |
| 3 | v1 | gradient bottom + cap | 2/3 (Opus); score inflation found |
| 4 | v2 | strict count table | g1 fixed; g2 still 3; floor ~50% (model noise, not regression) |
| 5 | v3 | few-shot for floor + inflation | floor 8/9, inflation 6/6, clean 2/2 |
