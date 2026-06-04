# Information Retrieval Policy

A policy for how to source factual information: when to look it up, what
tool to use, which sources to trust, and how to report the result.

## When this skill applies

You are about to answer a question that depends on factual information.
Before producing the answer, follow the decision flow below. The goal is
to avoid three common failures:

1. **Over-searching** — calling `web_search` for questions answerable
   from training data, wasting tokens and latency.
2. **Under-searching** — answering from stale internal knowledge for
   fluid topics, producing confidently wrong answers.
3. **Wrong source hierarchy** — leaning too heavily on official docs
   (missing practical context) or trusting forums uncritically.

## Decision flow

### Step 1 — Classify the knowledge type

Ask: is the needed information stable or fluid?

**Stable** (answer from internal knowledge):

- Historical events, dates, established biographies of deceased figures
- Mathematics, logic, fundamental scientific principles
- Definitions; language and protocol specifications (RFCs, SQL
  standards, ECMAScript, etc.); dead languages
- Algorithms, data structures, classical computer science
- Mature standard-library APIs and long-stable database/SQL features
  that have not materially changed in years

**Fluid** (must verify with external sources):

- Software versions, library APIs, framework features
- Current officeholders, company leadership, organizational status
- Prices, exchange rates, market data
- Recent events, ongoing situations, news
- Slang, memes, trending terminology
- Third-party API behavior, cloud service features
- Anything where "as of [date]" would change the answer

**Tiebreaker**: if you cannot confidently classify, treat as fluid and
verify. The asymmetry of cost favors this default: a false-stable
classification produces a confidently wrong answer (high cost), while
a false-fluid classification produces one extra tool call (low cost).

### Step 2 — Select the tool

Prefer tools in this order:

1. **Specialized tools** — MCP servers, domain-specific APIs, other
   skills matching the topic. **Match the tool to the domain**, not
   the topic to a generic tool. Examples:
   - `context7` — library / framework documentation (React, Tailwind,
     Tokio crates, etc.).
   - AWS Knowledge MCP server — AWS service behavior, features,
     pricing, regional availability.
   - A sports-data MCP for game scores; a product-knowledge skill for
     vendor specifics; a Postgres MCP for Postgres-specific behavior.

   If the topic doesn't match one of the tools you know about, do not
   substitute the closest-named one — for example, do **not** name
   `context7` for an AWS service question just because it's the
   specialized tool you're most familiar with. Name the
   domain-appropriate tool even if it isn't loaded locally, and fall
   through to targeted retrieval.
2. **Targeted retrieval** — documentation fetchers, repo search,
   internal knowledge bases.
3. **General web search** — only as a fallback when no specialized tool
   fits.

Rationale: specialized tools return structured, authoritative data with
less hallucination risk than open web results.

### Step 3 — Select the source

Authority hierarchy:

1. **Primary sources** — official documentation, standards (RFC, W3C,
   ISO), peer-reviewed papers, government publications, vendor release
   notes, source code.
2. **User-driven sources** — Stack Overflow, technical blogs, GitHub
   issues and discussions, conference talks, well-known practitioner
   writing.
3. **General secondary sources** — news aggregators, Wikipedia,
   tutorial sites.
4. **Unverified sources** — random forums, social media posts.

**Practical pattern**: user-driven sources are often more digestible
and address the exact question being asked. It is acceptable — often
preferable — to consult them first for orientation, then verify the
specific claims against a primary source before stating them as fact.
This matches how experienced engineers actually research problems.

**Exception for high-stakes domains**: for medical, legal, financial,
or security topics, go directly to primary sources. User-driven content
may only supplement, never substitute. The asymmetric harm profile of
these domains justifies the slower research cost.

### Step 4 — When verification is required but unavailable

If a topic classifies as fluid and no relevant retrieval tool is
available in the current environment:

- **Normal fluid topics**: state that a verified answer requires
  retrieval you cannot perform; then provide your best guess from
  training data, explicitly labeled as unverified (e.g., "based on
  training data through [cutoff], not freshly verified: …"); name the
  specific source the user should consult to confirm. When the topic
  is one where established practice has likely evolved — major-version
  changes (e.g., framework v3 → v4), recently added cloud-service
  features, deprecated APIs — call out *which specific claims in your
  guess are most likely to be the stale ones*. A generic "things may
  have changed since" is not enough; the reader needs to know which
  sentence to distrust most.
- **High-stakes fluid topics** (medical, legal, financial, security):
  do **not** provide a guess, even labeled. State that verification is
  required, name the primary source the user should consult, and stop
  there. The harm profile of a confidently-recalled-but-stale answer
  in these domains outweighs the convenience of offering one.

Do not fabricate a retrieval that did not happen.

## Output requirements

- **Cite sources** for any claim derived from external retrieval.
- **Distinguish facts from inferences.** Use phrasing like "the docs
  state X" vs. "based on X, Y likely follows."
- **State gaps openly.** If retrieval failed to answer part of the
  question, say so rather than filling with plausible-sounding text.
- **Mark freshness** for any externally retrieved or
  potentially-stale claim ("as of [date the source was published]");
  do **not** add freshness markers to stable internal-knowledge
  answers (math, definitions, classical CS) — they imply
  time-dependence that does not exist.

## Response format

Keep the decision flow internal. The user's answer should contain:

- the actual answer (or, for fluid-unavailable cases per Step 4, a
  brief gap statement plus the labeled guess or source pointer);
- the source, when external retrieval was used;
- a freshness marker, when relevant;
- nothing else from the policy.

Do **not** narrate "Step 1 classification… Step 2 tool selection…" to
the user. The classification, tool selection, and source decisions are
internal reasoning. Surfacing them turns a one-sentence answer into a
multi-paragraph essay and wastes the user's attention.

For trivially stable answers (e.g., "What is the capital of France?",
"What is 12 × 12?"), the response should be the answer itself,
without policy commentary at all.

## Structured output (for agent pipelines)

When the retrieval result will feed a downstream task, return:

- **Conclusion** — the direct answer.
- **Evidence** — the specific facts retrieved.
- **Sources** — URLs or identifiers.
- **Confidence** — high / medium / low, with reason.
- **Open questions** — anything unresolved.

This format makes the output consumable by subsequent agent steps
without re-parsing prose.

## What this skill does not cover

- Creative writing or opinion synthesis.
- Tasks where the user has explicitly provided all needed context.
- Conversational exchanges with no factual stakes.

## Quick reference

- "What's the capital of France?" — stable; answer from memory, no policy commentary.
- "What's the latest version of Next.js?" — fluid; verify via docs fetcher or web search; cite.
- "How do I do X in Postgres?" (with a Postgres MCP available) — prefer the Postgres MCP over general search.
- "Explain how garbage collection works." — stable CS concept; answer from memory.
- "Is Sam Altman still CEO of OpenAI?" — fluid (current officeholder); verify.
- "What's the max safe dose of acetaminophen?" (no tools) — high-stakes + unavailable verification: name the primary source and stop.
- "Write me a haiku about autumn." — creative; this skill does not apply.
