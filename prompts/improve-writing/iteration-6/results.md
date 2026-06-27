# Iteration 6 — candidate-v4 (Intent Check section + Observation Layer tags)

**Eval sets:** eval-set-v1 (regression subset), eval-set-v2 (gradient), eval-set-v3 (new feature cases).
**Candidate model:** Opus (matches the certification model of iterations 4–5).
**Judge model:** Opus, reference-guided binary verdicts (must_do / must_not_do).

## What changed in candidate-v4

candidate-v4 = candidate-v3 + two additions that separate the three feedback
layers the user identified (Idiom / Style / Intent):

1. **New `## Intent Check` section** (between Errors and Observations). Surfaces
   *meaning-altering* suggestions — the "is this what you actually meant?" layer —
   instead of discarding them silently or smuggling them in as idiom Observations.
   Eligibility is **grounded only** (a stated-intent contradiction in mixed mode,
   or a near-pair slip); default is `No intent concerns.` It is **excluded from the
   Naturalness Score** (like Errors) and **not applied to the Revised Sentence**.
2. **Per-Observation `Layer` tag** — header is now
   `### N. 💡 <Impact> · <Layer> — <pattern>`, where Layer ∈ {`Idiom` (learn it),
   `Style` (taste)}. Layer is descriptive; the score is still by Impact only.
3. Role section gained a "three kinds of feedback" frame; reasoning checklist,
   output template, examples (incl. a new Example 7 demonstrating Intent Check),
   and anti-pattern list updated accordingly.

## Method

13 cases run in one batch, plus the user-supplied `proof-01` (the real-world
failure that motivated the change). Each case: one Opus candidate subagent
(subject-under-test, Read+Write only) → one Opus judge subagent (reference-guided
binary). Judges were told the Intent Check section and Layer tags are **expected
additions**, so the pre-feature v1/v2 rubrics (notably eval-1's section-order
criterion) would not false-flag them.

Cases:

- **Regression (score discipline + Intent-Check-must-not-misfire):** eval-1
  (native floor), eval-3 (bilingual error vs observation), eval-4 (meaning-drift
  trap), eval-7 (casual register), eval-12 (bilingual word-form), gradient-1/2/3
  (the historically weak score-discipline cases).
- **New feature (eval-set-v3):** v3-1/v3-2 (Intent Check *should fire* — grounded
  contradiction / antonym), v3-3 (conservatism + Idiom layer), v3-4 (conservatism
  in bilingual mode), v3-5 (Idiom + Style layer tagging), proof-01 (the
  `tendency → rationale` false-suggestion failure).

## Result: 14 / 14

| Case | Score | Intent Check | Verdict | What it proves |
| --- | --- | --- | --- | --- |
| eval-1 | 5 | empty | ✅ | native floor unchanged; new sections don't break order |
| eval-3 | 3 | empty | ✅ | agreement Error vs collocation Observation still split |
| eval-4 | 4 | empty | ✅ | meaning-drift trap: no `deep`, IC did **not** misfire |
| eval-7 | 5 | empty | ✅ | casual register preserved |
| eval-12 | 5 | empty | ✅ | word-form Error (meaning-preserving) stays out of IC |
| gradient-1 | 2 | empty | ✅ | pervasive light-verb → 2 |
| gradient-2 | 2 | empty | ✅ | 5-cap + low score held (the historically flaky one) |
| gradient-3 | 5 | empty | ✅ | errors orthogonal to score; floor = 5 |
| **v3-1** | 5 | **FIRED** | ✅ | grounded negation contradiction → Intent Check, score unmoved |
| **v3-2** | 5 | **FIRED** | ✅ | antonym (削減=reduce vs "increase") → Intent Check, score unmoved |
| v3-3 | 4 | empty | ✅ | light-verb → Observation tagged `Idiom`; IC stays empty |
| v3-4 | 5 | empty | ✅ | bilingual but faithful → no invented IC item |
| v3-5 | 4 | empty | ✅ | `Idiom` (light-verb) + `Style` (drop `actual`) tagged correctly |
| **proof-01** | 5 | empty | ✅ | `tendency → rationale` **not** offered as a neutral swap — failure fixed |

**Intent Check fired on exactly the 2 grounded-divergence cases and stayed empty
on the other 12** — including all three conservatism traps (eval-4, v3-4,
proof-01). No score regression: every regression case landed on its expected
score, and the new sections did not perturb the gradient score discipline.

### Note on the first v3-3 run (eval-case fix, not a candidate defect)

On the first pass v3-3 scored ❌ (13/14). Cause: my input
*"We **did** the validation … before we **save** it"* carried an incidental
past/present tense mismatch, which candidate-v4 correctly flagged as a tense
Error — legitimately violating the case's "No errors found" premise. The feature
under test had passed (light-verb → Observation, Layer `Idiom`, IC empty). I
corrected the input to present-tense (`We do the validation of …`) and re-ran:
✅. The defect was in the eval case, not the prompt.

## Conclusion

candidate-v4 adds the Intent Check layer and Observation Layer tags **without
regressing** any existing discipline, fires the new section only when grounded,
and fixes the motivating `proof-01` failure. Recommend promoting
`prompt-candidates/candidate-v4.md` → `prompt.md` (pending greenlight).

### Caveats

- Single run per case (no variance study). gradient-2 has been flaky across prior
  iterations (iteration-4 saw it stuck at 3); it scored 2 here, but a variance
  pass (3–5 reps on the score-discipline cases) would harden that claim.
- The v3 "should-fire" cases use unambiguous contradictions/antonyms. Softer,
  genuinely-ambiguous divergences (the harder, more realistic case) are not yet
  probed and may be where Intent Check is least reliable.
