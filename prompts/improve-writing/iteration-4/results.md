# Iteration 4 — candidate-v2 (strict count-based score) + variance study

## What changed in candidate-v2

The iteration-3 defect was score inflation at the low end (overlapping table
rows 2/3, no count guard). `candidate-v2.md` replaces §1 with a **strict,
ordered, count-based table** and an explicit **count guard**:

- Define M = surviving Medium/High count, H = surviving High count, L = Low count.
- First matching row wins: `M=0,L≤1 → 5`; `M=0,L≥2 or M=1 → 4`;
  `M=2–3 and H≤1 → 3`; `M≥4 or H≥2 or systemic → 2`; pervasive → 1.
- **Count guard:** "two or more Highs, or five or more Medium/High (incl.
  cap-omitted), is Score 2 — never 3."
- Example 2 was rewritten to a clean Score-4 (1 High + 1 Low) so the examples
  stop contradicting the table (v1's Examples 2 and 3 scored the *same* profile
  1H+1M as 4 and 3 respectively — a latent inconsistency).

Run on **Opus**: full eval-set-v1 regression (12) + eval-set-v2 gradient (3),
plus a **12-run variance study** to separate a real regression from model noise.

## Headline: the score is the weak link, and prompt rules can't fully fix it

candidate-v2 is **strictly better than v1 on the clear-cut case but does not
close the gap** — because the real problem is that **Opus does not treat the
Naturalness Score as mechanically derived.** It computes a *felt* score and
treats the §1 rules as soft, overriding them in both directions:

| Probe | v1 behavior | v2 behavior | Verdict |
| --- | --- | --- | --- |
| gradient-1 (3 High, short) | 3 (overlap-excused) | **2** (guard fired) | ✅ v2 fixed it |
| gradient-2 (M≥6, "clear" paragraph) | 3 (inflated) | **3** (guard *ignored*) | ❌ still broken |
| floor: errors-but-native → must be 5 | ~50% correct | ~50% correct | ⚠️ unfixed, not regressed |

### gradient-2: the explicit guard was overridden
v2 output: 3 High + 2 Medium + "(1 additional … omitted)" → M≥6, H=3. The §1
guard says *"H≥2 or M≥5 ⇒ Score 2, never 3."* The justification text even reads
*"systematically non-native"* — the literal Score-2 trigger. **Opus scored 3
anyway.** A crystal-clear deterministic rule, stated twice (table + reasoning
checklist), did not bind. The model has a strong prior that a *readable*
paragraph is "Clear but non-native (3)," not "Awkward (2)," and that prior wins.

### The floor ("errors do not affect the score") is ~50/50 on both versions
First v2 runs of eval-12 and gradient-3 came back 3/5 (0 phrasing observations,
so the rule mandates 5) — looking like a v2 regression. The variance study
refutes that:

| | orig | r1 | r2 | r3 | correct |
| --- | --- | --- | --- | --- | --- |
| eval-12 v1 | 5 | 3 | 5 | 4 | 2/4 |
| eval-12 v2 | 3 | 5 | 4 | 5 | 2/4 |
| gradient-3 v1 | 5 | 3 | 3 | 5 | 2/4 |
| gradient-3 v2 | 3 | 5 | 5 | 5 | 3/4 |

**Both v1 and v2 violate `M=0 ⇒ 5` about half the time** (scoring 3 or 4 because
the sentence "obviously has mistakes"). v2 is not worse — marginally better on
gradient-3. So this is a **model-level weakness, not a candidate-v2 regression.**

## Diagnosis

The model reliably does the **hard** part — finding and classifying phrasing
observations and their impact (M/H/L) — and unreliably does the **trivial**
part — mapping those counts to a number. It anchors on the score *label
semantics* ("Native-like" can't sit over errors; "Awkward" feels too harsh for
readable prose) and lets that override the arithmetic. No amount of rule-wording
fully removes that, as gradient-2 (explicit guard, still ignored) shows.

## Recommendation (the actual next step): move scoring out of the model

Play to the model's strength and remove its weakness:

1. **Compute the score in code.** Keep the prompt's job as *find + classify*
   observations (robust), have it emit the counts (or parse them from the
   Observations section), and derive `score = f(M, H, L)` programmatically. This
   makes the score match the observations **100%** of the time — eliminating
   both the gradient-2 inflation and the eval-12/gradient-3 floor violations in
   one move. Best fit given this prompt is served via the MCP handlers.

Alternatives if staying prompt-only:
2. **Add two few-shot examples** — an "errors-but-native → 5" case and a
   "pervasive → 2" case (the prompt currently has neither). Likely raises the
   obedience rate but, per gradient-2, will not reach 100%.
3. **Make the score advisory/banded** (Native / Needs polish / Heavy) and stop
   pinning exact values the model won't reliably produce.

## Regression detail (candidate-v2 × eval-set-v1, single Opus pass)

9 clean passes (eval-1,2,3,4,5,7,8,9,11) + eval-10 borderline-pass (fixed
`reason of`→`reason for` silently in the revision rather than surfacing it as an
explicit Observation; `totally`→`completely` justified on the allowed
collocation/semantic grounds). Two misses — eval-6 (restructured "the query who
is slow"→"the slow query" as a word-order Observation, never flagging
`who`→`that` as the required Error) and eval-12 (floor 3/5) — **both shown by the
variance study to be model noise present in v1 too**, not v2 regressions. Score
shifts eval-4 (5→4) and eval-10 (4→3) are expected and arguably more correct
under the deterministic table; both stay within their rubrics.

## What candidate-v2 is worth

Keep it: it is strictly better than v1 (fixes the High-heavy inflation
deterministically; examples are now self-consistent) and no worse on the floor.
But it is **not sufficient** — the residual inflation (gradient-2) and the
~50% floor violations need the **code-side score** (option 1). Prompt-tuning the
score has reached diminishing returns.

## Cumulative record

| Iteration | Prompt × Eval | Models | Result |
| --- | --- | --- | --- |
| 1 | v1 × v1 | Opus / Sonnet | 12/12 |
| 2 | v1 × v1 | Haiku / Opus | 11/12 (leniency) |
| 3 | v1 × v2 (gradient) | Opus / Opus | 2/3 (score inflation found) |
| 3 | v1 × v2 (gradient) | Haiku / Opus | 0/3 |
| 4 | v2 × v1 (regression) | Opus | 9 clean + 1 borderline + 2 noise |
| 4 | v2 × v2 (gradient) | Opus | g1 fixed (2); g2 still 3; g3 noise |
| 4 | variance (eval-12, g3) | Opus, v1 vs v2 | floor ~50% on **both** versions |
