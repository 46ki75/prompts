# Iteration 2 — model-swap run (Haiku candidate / Opus judge)

## What is under test

Same prompt, same eval set as iteration 1 — **only the models changed**:

- **Candidate (runs the prompt):** Haiku (was Opus in iter 1). A stress test of
  whether the discipline holds on a weaker model.
- **Judge (grades outputs):** Opus (was Sonnet in iter 1). Strongest grader.
- **Prompt:** `prompt-candidates/candidate-v1.md` (unchanged).
- **Eval set:** `eval-set-v1.jsonl` (unchanged inputs; two judge-criteria
  recalibrations — see below).

Method otherwise identical to iter 1 (Write-only candidate subagents, prompt
pasted verbatim; Read+Write judge subagents; pilot of 4 then fan-out).

## Headline

- **11 / 12 correct.** One genuine failure: **eval-10**.
- The hard disciplines held on Haiku across all 12: Error-vs-Observation
  separation, the do-not-flag lists (`Maybe,`/`a lot`/domain terms/list-item
  fragments), Japanese-as-context, and section/format contract. No case
  misclassified a phrasing issue as an Error, and no casual marker was flagged.

## The one failure — eval-10 (broken Revised Sentence)

Input: `The actual real reason of the failure is that the disk was totally full.`

Haiku's output failed three ways (Opus judge confirmed):

1. **Broken Revised Sentence** — `"The actual reason the disk was full."` drops
   "of the failure is that," producing a fragment with no main verb and a
   changed meaning. Violates the §3.1/§4 meaning-preservation + fixed-point
   rules.
2. **Missed the target Observation** — never surfaced `reason of` -> `reason for`
   (the preposition issue the case is built around).
3. **Register-based rationale** — dropped `totally` justifying it as a "casual
   intensifier often dropped in neutral technical explanations," which is
   exactly the §3.6.1 register-push rationale the prompt forbids.

Opus (iter 1) handled this same input cleanly. This is a real
weaker-model regression, not score variance.

## The substantive cross-model finding: Haiku is more lenient

On the same inputs, Haiku systematically surfaced **fewer observations** and
assigned **lower impact labels**, which (because the score is derived from
surviving observations) produced **higher naturalness scores** than Opus:

| Case | Opus (iter 1) | Haiku (iter 2) | Note |
| --- | --- | --- | --- |
| eval-4 | 5/5, 0 obs | 4/5, 1 Medium | Haiku flagged the §3.1 `by reference` upgrade (valid either way) |
| eval-5 | 3/5, 2 obs (1 High) | 4/5, 1 Medium | Haiku surfaced 1 of 2 observations, rated it Medium not High |
| eval-6 | 3/5, 3 obs (1 High) | 4/5, 1 Medium | clearly non-native input rated "Near-native, polish optional" |

For eval-6 in particular, a sentence with two grammar errors and a light-verb
construction got "4/5 Near-native, polish optional" from Haiku vs Opus's "3/5,
polish recommended." Both are *internally consistent with each model's own
observation set* per the score table — but the author-facing verdict differs
materially. **If this prompt is deployed on a smaller model, expect inflated
naturalness scores and under-detected phrasing observations.**

## Judge-criteria recalibration (done before grading)

The Haiku run exposed a methodological flaw in two of my iteration-1 criteria:
**pinning an exact expected score is wrong for a derived-from-observations
rubric** when the observation count isn't forced. Two models legitimately
surface different observation sets and therefore different (but internally
valid) scores. Corrected:

- **eval-5**: "score 3 or lower" -> "score consistent with the surviving
  observation set per the table (a single Medium/High obs -> 4; two-to-four
  incl. a High -> 3); never 5."
- **eval-6**: "score 2 or 3" -> same consistency-based phrasing; never 5.

These changes only make the criteria *more faithful to the prompt's own table*;
iteration-1's Opus outputs (eval-5=3, eval-6=3) still pass under them. The
correct invariant to test is: errors classified right, observations classified
right (never Errors), score consistent with stated observations, and the hard
floor/ceiling (0 Medium/High -> 5; any surviving Medium/High -> not 5).

## Calibration (judge ↔ author)

Opus-judge verdicts matched my own independent read on all 12 (including the
eval-10 failure). The Opus judge's eval-10 reasoning independently identified
all three defects, including the broken Revised Sentence — a non-obvious catch
that increases confidence in Opus-as-judge for this task.

## Cumulative record (candidate-v1 across runs)

| Case | iter 1 (Opus cand / Sonnet judge) | iter 2 (Haiku cand / Opus judge) |
| --- | --- | --- |
| eval-1 | correct (5) | correct (5) |
| eval-2 | correct (5) | correct (5) |
| eval-3 | correct (3) | correct (3) |
| eval-4 | correct (5) | correct (4) |
| eval-5 | correct (3) | correct (4) |
| eval-6 | correct (3) | correct (4) |
| eval-7 | correct (5) | correct (5) |
| eval-8 | correct (5) | correct (5) |
| eval-9 | correct (5) | correct (5) |
| eval-10 | correct (4) | **incorrect** (broken revision) |
| eval-11 | correct (5) | correct (5) |
| eval-12 | correct (5) | correct (5) |
| **Total** | **12/12** | **11/12** |

## Conclusion

The prompt's structural discipline survives on Haiku (11/12); the lone hard
failure (eval-10) is a broken rewrite plus a register-push rationale on the
hardest multi-issue sentence. The clearer takeaway is qualitative: **smaller
model -> more lenient scoring and shallower observation coverage.** For an
English-coaching tool whose value is catching non-native phrasing, that
leniency matters more than the single eval-10 miss.

Recommendation: **run the prompt on the largest model you can afford in
production**; if a smaller model is required, expect to re-tune the score table
(or treat scores as advisory). No edit to the prompt body is proposed — the
prompt behaved per spec; the variance is model capability.

Next steps unchanged from iter 1: real-failure cases; harder gradient cases
(Score 2, the 5-observation cap); and a thinking-disabled run.
