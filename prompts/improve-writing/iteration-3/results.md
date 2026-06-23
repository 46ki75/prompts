# Iteration 3 — gradient stress test (bottom of the score scale + the cap)

## What is under test

`candidate-v1.md` (unchanged prompt) against a new focused probe set,
`eval-set-v2.jsonl` (3 cases), targeting the cells iterations 1–2 never
exercised:

- **gradient-1** — Score 2 region: grammatical but systemic nominalization /
  light-verb. Probes the Score-2/3 boundary and Error-vs-Observation discipline.
- **gradient-2** — the **5-Observation cap** (§3.3): force `>5` surviving
  observations so the model must cap at 5 and append
  `"(N additional … omitted.)"`. Bottom-of-gradient score.
- **gradient-3** — the **orthogonality rule** at its extreme: an error-riddled
  sentence whose *corrected phrasing* is fully native → spec says **5/5** with
  every error in §2. Tests "errors do not affect the score."

Each case run on **both Opus and Haiku** candidates (to separate *prompt
capability* from the *model-leniency* finding of iteration 2), graded by
**Opus** judges against per-case `must_do`/`must_not_do`.

## Headline

| Case | Opus candidate | Haiku candidate |
| --- | --- | --- |
| gradient-1 (Score 2) | ✅ correct (3/5) | ❌ meaning drift in revision |
| gradient-2 (cap) | ❌ score inflated (3/5) | ❌ light-verb filed as Error |
| gradient-3 (orthogonality) | ✅ correct (5/5) | ❌ errors double-counted, scored 3/5 |
| **Total** | **2 / 3** | **0 / 3** |

The gradient set did its job: it broke things that iterations 1–2 (easier
cases) never touched. Two findings matter.

## Finding 1 (substantive, prompt-level): the score is *not* reliably "derived" at the low end

The prompt calls the Naturalness Score "derived, not felt." At the **bottom**
of the scale that derivation leaks:

- **gradient-2 / Opus** capped correctly and even printed
  `"(1 additional lower-impact observation omitted.)"` — i.e. it *declared* that
  **≥6 Medium/High observations survived** — then assigned **Score 3**. The
  table caps Score 3 at **"2–4 total"**; 5+ surviving Medium/High observations
  can only be Score 2 or 1. The model surfaced the very data that forbids 3 and
  scored 3 anyway.
- **gradient-1**: Opus (1 High + 2 Medium) → 3, clean. Haiku rated all three
  observations **High** but still scored **3** — which the Score-2 row
  ("3+ with multiple Highs") arguably mandates as 2.

Root cause is the **table itself**, not just the model:

1. **Rows 2 and 3 overlap.** "3 observations, all High" satisfies *both*
   row-3 ("2–4 total with ≥1 High") and row-2 ("3+ with multiple Highs"). No
   tie-breaker → models resolve upward to the friendlier "Clear but non-native."
2. **The count bound is buried.** The "2–4 total" ceiling for Score 3 lives in a
   table cell with no hard rule like *"if ≥5 Medium/High survive, score ≤2."*
   So when the cap note says "6 survived," nothing forces the recount.
3. Result: models **anchor at Score 3** and resist 2/1 — exactly the "felt, not
   derived" failure the section title warns against.

### Proposed fix (candidate-v2)

Replace the overlapping table with a strict, ordered, mutually-exclusive
decision on `M` = surviving Medium/High count and `H` = surviving High count:

```
M == 0            -> 5
M == 1            -> 4
M in 2..3, H <= 1 -> 3
M in 2..3, H >= 2 -> 2
M >= 4            -> 2   (1 if pervasive High-impact impedes reading)
```

Plus one hard rule: **"If ≥5 Medium/High Observations survive (counting any
omitted by the 5-cap), the score is at most 2."** This removes the row-2/3
overlap and makes the count bound bite. (Note: this is a *prompt* change, so it
would be `candidate-v2` and a fresh run — not edited silently.)

## Finding 2: Haiku is unreliable on this task — 0/3, three distinct core-discipline breaks

Each Haiku failure is a *different* invariant, which is worse than one repeated
slip:

- **gradient-1** — Revised Sentence *changed the meaning*:
  `"We implemented the system and tested it…"` drops "the feature" and swaps the
  implemented object (feature → system), collapsing two distinct objects into
  one. Violates §3.1 meaning-preservation + the fixed-point rule.
- **gradient-2** — Filed the light-verb `"do the notification of" → "notify"`
  in the **Errors** section as "wrong light-verb valency." Light-verb is the
  single most-tested NOT-an-Error category (§2). The misfile also suppressed the
  overflow note (only 5 observations remained).
- **gradient-3** — Listed the *same two grammar fixes in both Errors and
  Observations* (`user click→clicks`, `request are send→is sent`), then used
  those duplicate "observations" to drag the score to **3/5**. Breaks two rules
  at once: "errors do not affect the score" and "an issue is either an Error or
  an Observation, never both."

Combined with iteration 2 (11/12 but with the eval-10 break and systematic score
inflation), the cumulative verdict on Haiku is clear: **do not run this prompt
on Haiku in production.** It does not hold the scoring model's invariants once
inputs get hard.

## Finding 3: the cap/overflow mechanic works — but only with enough material

The first gradient-2 input was a **single sentence** with ~6 candidate issues.
Both models **naturally consolidated to exactly 5** observations and never
triggered the overflow note (Opus even silently applied a 6th fix in the revised
sentence without listing it). Preserved as
`candidate-v1-*-attempt1-singlesentence.txt`.

Strengthening the input to a **4-sentence paragraph** with ≥6 distinct issues
made Opus exercise the path correctly: exactly 5 shown, ranked High→Medium,
`"(1 additional lower-impact observation omitted.)"` appended, and the casual
marker `"Basically,"` correctly left unflagged. **Usage note:** the 5-cap is
effectively a multi-sentence / paragraph feature; single sentences rarely yield
>5 surviving observations.

## Finding 4 (design question for you): 5/5 on an error-riddled sentence

gradient-3 is correct *per spec* — naturalness is orthogonal to grammar, so a
sentence with 3 grammar errors but native phrasing scores **5/5** (Opus did
exactly this, adding the caveat "Once the grammar slips are corrected, the
phrasing is standard idiomatic technical English"). It's internally consistent,
but a user seeing **"5/5 — Native-like"** directly above three ⚠️ errors may be
confused. Options: (a) leave as-is (the one-line justification mitigates); or
(b) add a presentation rule — when Errors exist, phrase the score line as
"naturalness of the corrected phrasing." Your call; minor.

## Calibration (judge ↔ author)

Opus-judge verdicts matched my own independent read on **all 6** outputs. On
gradient-1/Haiku the judge surfaced a defect I had under-weighted (the
meaning-drift in the revision, not just the score) — an adversarial catch that
raises confidence in Opus-as-judge. Judge JSON: `gradient-N/judge-opus-of-*.json`.

## Conclusion

- The prompt's **format/section discipline survives** even at the gradient
  bottom on a capable model (Opus): cap works, discourse markers stay unflagged,
  orthogonality holds, meaning is preserved.
- The **one real prompt weakness** found this iteration is the **score
  derivation at Score ≤2**: overlapping table rows + no hard count-guard let even
  Opus inflate a 6-observation paragraph to Score 3. This is the first
  change-worthy defect across all three iterations. Fix = the strict
  count-based table above (candidate-v2).
- **Haiku remains unfit** for this prompt (0/3 here, three distinct invariant
  breaks).

## Cumulative record

| Iteration | Prompt × Eval set | Models (cand / judge) | Result |
| --- | --- | --- | --- |
| 1 | candidate-v1 × eval-set-v1 | Opus / Sonnet | 12/12 |
| 2 | candidate-v1 × eval-set-v1 | Haiku / Opus | 11/12 (leniency finding) |
| 3 | candidate-v1 × eval-set-v2 (gradient) | Opus / Opus | 2/3 |
| 3 | candidate-v1 × eval-set-v2 (gradient) | Haiku / Opus | 0/3 |

## Suggested next step

Implement **candidate-v2** (strict count-based score table + the ≥5→≤2 hard
rule) and re-run eval-set-v1 (regression: must stay 12/12 on Opus) **and**
eval-set-v2 (must now hit Score ≤2 on gradient-1/gradient-2). That would confirm
the fix removes the inflation without disturbing the passing cases.
