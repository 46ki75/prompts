# Iteration 1 — improve-writing, post-`<thinking>`-removal baseline

## What is under test

- **Candidate:** `prompt-candidates/candidate-v1.md` (verbatim snapshot of
  `prompt.md`).
- **Change being validated:** the in-band `<thinking>` block was removed and
  its reasoning + checks were re-expressed as a "Reasoning (think before you
  answer)" section meant to be consumed by the model's native/extended
  thinking. This iteration asks: **did dropping the scratchpad regress the
  scoring/filtering discipline?**
- **Eval set:** `eval-set-v1.jsonl` (12 cases).
- **Candidate model:** inherited session model (Opus) with native thinking.
- **Judge model:** Sonnet (different family tier than the candidate, to limit
  self-enhancement bias). One isolated binary judge per case, reasoning-then-
  verdict, graded against per-case `must_do` / `must_not_do`.

## Method

- Each candidate run: a `general-purpose` subagent, tools restricted to Write,
  prompt pasted verbatim inside `<prompt_under_test>`, eval input inside
  `<user_message>`, full answer written to `eval-N/candidate-v1.txt`.
- Pilot of 4 (eval-1, 2, 3, 5 — the highest-risk discipline checks) eyeballed
  before fanning out the remaining 8.
- Each judge: a `general-purpose` Sonnet subagent (Read + Write only) grading
  one output to `eval-N/judge-v1.json`.

## Headline

- **12 / 12 correct**, single sweep.
- **No regression detected from the `<thinking>` removal.** With native
  thinking, the model held the Error-vs-Observation boundary, the score
  derivation, the do-not-flag lists, register preservation, and the
  meaning-preservation filter — all the discipline that previously lived in
  the explicit scratchpad.

## Coverage (what each case probed)

| Case | Probe | Result |
| --- | --- | --- |
| eval-1 | Score-5 floor on clean native input | 5/5, no invented obs ✓ |
| eval-2 | Discourse marker + casual intensifier not flagged; article IS Error | `Maybe,`/`a lot` untouched, article flagged, 5/5 ✓ |
| eval-3 | JP+EN; agreement=Error, collocation=Observation | boundary clean, JP used as context only ✓ |
| eval-4 | Meaning-drift trap (no deep/shallow) | no precision added, 5/5 ✓ |
| eval-5 | Light-verb → High Observation (not Error); ≥1 High caps score | light-verb as High obs, Score 3 ✓ |
| eval-6 | Multi-error; errors must NOT lower score | both errors flagged, Score 3 (correct per table) ✓ |
| eval-7 | Casual notes register preserved | no formalization, 5/5 Casual ✓ |
| eval-8 | Domain terms not flagged | `immutable`/`idempotent` left alone, 5/5 ✓ |
| eval-9 | Headings/imperative list items not flagged | 5/5, no flags ✓ |
| eval-10 | Preposition=Observation; `totally` not formalized for register | `reason of` as obs (collocation), 4/5 ✓ |
| eval-11 | Self-doubt/taste obs discarded; score-5 floor | no invented obs, 5/5 ✓ |
| eval-12 | JP+EN; noun-for-adjective word-class slip = Error | `idempotency`→`idempotent` as Error, 5/5 ✓ |

## Failure modes

None this iteration.

## Calibration notes (judge ↔ author)

- The judge verdicts matched my own independent read on all 12 cases. The
  rubric criteria are concrete (specific section placement, score ranges,
  named phrases), which makes the judge robust but also means a high pass
  rate is partly a property of well-specified criteria, not only prompt
  quality.
- **One author-side miscalibration was caught during the pilot review and
  fixed before grading** (this is the calibration step working, not a prompt
  failure):
  - **eval-6** originally demanded "score 2 or lower." The prompt's explicit
    rule is *"Errors do not affect the score."* The two grammar errors are
    therefore score-neutral; the surviving phrasing observations (1 High +
    1 Medium + 1 Low) map to **Score 3** per the §1 table. The candidate's
    3 was correct; the rubric was wrong and was corrected to "2 or 3."
  - **eval-10** criterion was loosened so the `preposition` vs `collocation`
    pattern label is not graded (both are valid §3.4 patterns; the
    load-bearing requirement is that it stays an Observation, not an Error),
    and to clarify that dropping `totally` on a concision/semantic rationale
    ("full is absolute") is acceptable — only a *register-only* rewrite is a
    violation.

## Caveats / known gaps

1. **Synthetic eval set.** All 12 cases were hand-authored to probe stated
   discipline, not sourced from real logged failures. 12/12 on a synthetic
   set is encouraging but weak evidence on its own; replace/augment with real
   author inputs as they accumulate.
2. **Thinking-on dependency not isolated.** Candidates ran on Opus with native
   thinking available — which is the whole point of the refactor. The prompt
   has **not** been tested in a no-thinking / thinking-disabled config, where
   the relocated reasoning has no channel to run in. If the prompt will ever
   be used without extended thinking, that config needs its own run before
   trusting these results.
3. **No adversarial/position-swap judging.** Single binary judge per case (not
   pairwise), so no first-position bias to control, but also no second
   opinion per case.

## Conclusion

The `<thinking>` → native-thinking refactor is **behaviorally clean on this
set — 12/12, no discipline regression**. No prompt edit is proposed for
iteration 2. Recommended next steps if continuing:

- Add real-world author inputs and harder gradient cases (Score 2, cap-of-5
  overflow, the "(N additional … omitted.)" path).
- Run one batch with thinking disabled to confirm the relocated reasoning
  still holds (or document that thinking-on is a requirement).
