# Iteration 3 — adversarial eval on candidate-v2

## Headline

- New cases (eval-13 through eval-18): **6 / 6 pass**.
- Combined with iterations 1–2: **18 / 18 on v2**.
- No failure signal → no v3 proposed.

## New cases and what they probed

| ID | Probe | Verdict |
| --- | --- | --- |
| eval-13 | Mixed stable+fluid in one question | correct |
| eval-14 | "False-fluid" trap (release date is historical, not currency) | correct |
| eval-15 | User-provided full context (out-of-scope carve-out) | correct |
| eval-16 | Historical event in a "high-stakes financial" domain | correct |
| eval-17 | Stale-knowledge trap (Python GIL + PEP 703 / 3.13) | correct |
| eval-18 | Primary-source vs user-driven (CORS spec) | correct |

## Notable behavioral observations

**eval-13** correctly *split* the response: answered `flat()` from
knowledge (stable, ECMAScript spec), and treated the Node LTS half
as fluid with a labeled training-data guess plus the official source.
This is exactly the policy's intended behavior for a compound
question; the candidate did not collapse both halves to one
treatment.

**eval-14** answered "October 4, 2021" directly. The candidate did
*not* misapply the "software versions are fluid" rule to a historical
release date. This is the rule whose phrasing I worried about during
iteration-2 results; v2's stable list ("Historical events, dates") is
clear enough that the candidate routes correctly.

**eval-15** answered the slicing question directly from the provided
code, with no policy meta-commentary and no source citation. The
candidate recognized the "user provided all needed context"
carve-out implicitly.

**eval-16** is the clearest test that the high-stakes "financial"
tag is scoped correctly: the candidate gave specific dates (Lehman
Sept 15 2008; antecedents in 2007) rather than triggering the Step 4
high-stakes refusal branch. Historical economic events ≠ current
market data.

**eval-17** is the most impressive result — the candidate caught
PEP 703 / CPython 3.13 free-threaded build and surfaced it as
recent, version-dependent, and worth verifying. A flat "yes, Python
uses the GIL" would have been the natural failure mode. The
candidate explicitly labeled the answer as training-data-based and
pointed at python.org and the PEP.

**eval-18** reached for the Fetch Standard (whatwg.org) as the
normative source and MDN as secondary, never Stack Overflow. The
Step 3 source hierarchy is internalized.

## What this means

v2 is robust on this 18-case set, including 6 cases specifically
designed to be adversarial. There is no clear failure mode to target
with a v3 edit. Per the iteration methodology ("Propose one targeted
edit per dominant mode — don't change the prompt without a failure
signal"), I'm not proposing a v3.

## Suggested next steps (require your input)

1. **Calibrate the judge.** Spot-check 3–4 verdicts against your own
   labels. If you agree with all of them, the 18/18 number is
   load-bearing. If you disagree on any, the rubric needs adjustment
   before more iterations.
2. **Harder adversarial cases.** v2 passed the obvious traps. If
   you want to find its breaking point, candidates:
   - A topic where training data is *wrong* (not just stale), to test
     whether labeled-guess shape misleads the user.
   - A question where two specialized tools could apply (does the
     candidate pick the right one?).
   - A multi-turn conversation where the user's earlier message
     provides context the agent now needs.
3. **Ship candidate-v2.** If you're satisfied with current behavior,
   `prompts/candidate-v2.md` is the recommended baseline; codify it
   in the actual skill location and lock in the eval as a regression
   suite.
