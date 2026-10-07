# User memory index

Cross-project facts and preferences learned while working for this user.

- **Reply language: English.** The user may write in Indonesian, but wants
  answers in English. Do not switch to another language (e.g. Chinese) on your
  own.

- **Sandbox has no `node_modules` and no `pnpm`.** To verify work in a pnpm
  monorepo here without a full install: `npx --yes prettier@<v> --check|--write`,
  `npx --yes esbuild@<v> --loader:.ts=ts <file>` to parse-check, and for tests
  `npm i vitest@<v>` in a scratch dir + a scratch `vitest.config.ts`
  (`environment: "node"`, absolute `include`) then
  `vitest run --root <repo> --config <scratch>`. `npx --yes tsx@<v> <file>` runs a
  single TS module. Used 2026-10-05 on cipansor PR #512 — details in
  [2026-10-05.md](./2026-10-05.md).

- **2026-10-05 — AWS account audit & cleanup.** User has a separate personal/foundation AWS
  account (not the cipansor repo, which is Azure). Audited read-only then cleaned up with
  root `aws login --remote`. Durable how-tos (CLI v2 install, `aws login --remote` headless
  via FIFO, `timeout` per region, IAM-user/permission-set delete order, Resource Explorer as
  the cause of "everything looks used") and the account state are in
  [2026-10-05.md](./2026-10-05.md). No secrets recorded there.
  Open follow-ups: enable CloudTrail/GuardDuty/Config/AccessAnalyzer, set password policy,
  add alternate contacts, revoke the root CLI session.
- **Kiro Enterprise cannot be subscribed until the account is VERIFIED.** Attempted end-to-end
  on 2026-10-05; console returned "account is not currently eligible to create new
  subscriptions" (contact AWS Support). Proof: `aws account get-contact-information` ->
  `VerificationStatus: UNVERIFIED`, even though the org/account state is ACTIVE. Account
  "active" != "verified". Console path is `/amazonq/developer/home#/kiro/users`. Console
  federation works only with `sts get-federation-token` (root/`get-session-token` creds are
  rejected: "Only federation tokens or assume role tokens"). No IAM/credit workaround.
  Details in [2026-10-05.md](./2026-10-05.md).

## deepseek-v4.1-flash vision (investigated 2026-10-05) — TWO distinct mechanisms
Root cause lives in **software-agent-sdk**, not this Canvas repo. Canvas has no model-level
vision list; the Agent Server/SDK decides, so any Canvas "vision-capable profile" list is a
mirror of SDK `supports_vision`.

**A. Image DROP (all deepseek).** `FORCE_STRING_SERIALIZER_MODELS` in
`openhands-sdk/openhands/sdk/llm/utils/model_features.py` has the bare substring `"deepseek"`,
so every DeepSeek id resolves `force_string_serializer=True`; `Message.to_chat_dict()` then
takes the string branch before the `vision_enabled` branch and silently drops `ImageContent`.
Covered: issue #5360 + 3 open PRs (#5467 approved/CLEAN, #5460 approved/UNSTABLE, #5367 draft).
#5360 assumes `vision_is_active()==True`, so it does NOT cover mechanism B.

**B. Not DETECTED as vision-capable (specific to deepseek-v4.1-flash).** `get_features(...)`
resolves `supports_vision` as override -> proxy `model_info.supports_vision` -> fallback
`litellm_supports_vision(_normalize_model_for_litellm(model))`. LiteLLM's live registry
(`BerriAI/litellm@main`) has NO entry for `deepseek-v4.1-flash` / `deepseek/deepseek-v4.1-flash`
-> False. `deepseek-v4-flash` is True and `kimi-k3` is True because the SDK special-cases
`kimi-k3 -> moonshot/kimi-k3` in `_normalize_model_for_litellm` and lists it in
`VISION_MODEL_OVERRIDES`/inline-image lists. `VISION_MODEL_OVERRIDES` is `{}` for deepseek.
The model IS vision-capable elsewhere: LiteLLM has `baseten/deepseek-ai/DeepSeek-V4.1-Flash`
supports_vision=True. Consequence: `_candidate_vision_profiles()` in
`tool/builtins/vision_inspect.py` (uses `llm.vision_is_active()`) omits the v4.1-flash profile,
so the auxiliary vision tool routes to kimi-k3.
Natural fix (SDK): add `deepseek-v4.1-flash` to the vision-capable set; broader fix is
umbrella #4880 (own capability metadata, don't outsource to LiteLLM main).

**Second fix in same PR (2026-10-05, commit 53adb22):** Even after B is fixed,
mechanism A still dropped the image, because the bare `"deepseek"` substring in
`FORCE_STRING_SERIALIZER_MODELS` resolves `force_string_serializer=True` and
`Message.to_chat_dict` takes `_string_serializer()` before the `vision_enabled`
branch. Key gotcha: **`model_matches` does NOT honor a `!` prefix** — only
`apply_ordered_model_rules` does (last-match-wins include/exclude). So the fix
had to (1) add `!deepseek-v4.1-flash` / `!deepseek-v4-flash` after `"deepseek"`
and (2) switch the `force_string_serializer=` resolution in `get_features` from
`model_matches(...)` to `apply_ordered_model_rules(...)`, mirroring
`PROMPT_CACHE_RETENTION_MODELS`. `message.py` itself untouched (that is #5360).
Open item: DeepSeek list-format compatibility for `tool` messages is NOT verified
by a live request.

**Outcome (2026-10-05):** Created issue **OpenHands/software-agent-sdk#5523** and PR
**OpenHands/software-agent-sdk#5524** (draft) for mechanism B.
Fix verified locally (branch `fix/deepseek-v4.1-flash-vision-capability`, pushed to
`analisaperlengkapan/software-agent-sdk`; patch also saved at
`.openhands/pr-5523/deepseek-v4.1-flash-vision.patch`).
Design: add `VISION_CAPABLE_MODELS: list[str] = ["deepseek-v4.1-flash"]` to
`model_features.py` and consult it as a fallback in `_model_supports_vision` after the LiteLLM
check. Deliberately did NOT re-add to `VISION_MODEL_OVERRIDES`: PR #4567 removed the kimi-k3
entry from there and made `test_vision_overrides_are_not_redundant` fail if any entry points at
a LiteLLM-resolvable id. Tests: 1173 passed in `tests/sdk/llm/`, ruff clean, patch applies
cleanly to upstream `main`.
**Token constraint learned:** the GH App token (`analisaperlengkapan`, `ghu_` prefix) has
`push:false` on upstream `OpenHands/*` repos and cannot fork (`POST .../forks` -> 403
"Resource not accessible by integration") or create repos. It CAN push to repos the user
already owns (e.g. `analisaperlengkapan/OpenHands`, `analisaperlengkapan/software-agent-sdk`
once the user forked it). So for upstream PRs: ask the user to fork first, then
`git remote add fork https://x-access-token:$GITHUB_TOKEN@github.com/<user>/<repo>.git`,
push the branch, and `gh pr create --repo <upstream> --head <user>:<branch>`.
PR kept as **draft** because the repo requires a human-written `HUMAN:` section before
`ready_for_review`; AI must not fill it. PR body drafted at `.openhands/pr-5523/pr_body.md`.
Canvas feature request (different, auto-routing): OpenHands/OpenHands#16679.

- **2026-10-06 — OpenHands competitive analysis + filed issues.** Profiled Kiro/Kiro Crew,
  OpenClaw, CrewAI, Devin, Jules (and their repos) vs all OpenHands repos. Filed 5 feature
  issues (OpenHands#18054–#18058: proactive mode, messaging gateway, hardened sandbox,
  live agent activity view, memory retrieval) and posted evidence comments on #16315, #17691,
  SDK#4254/#4829/#4251. Key correction: persistent `MEMORY.md` already ships (SDK#4178/#4205,
  OpenHands#16097). GH App token can create issues but NOT apply labels (403). Details in
  [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 (live activity view) + PR-validator gotcha.** Opened
  `OpenHands/OpenHands#18064` (draft) for the user's own `ready-for-dev` issue
  #18057; branch `feat/live-activity-view` on fork `analisaperlengkapan/OpenHands`.
  `check_pr_description.py`: a Feature PR must link an issue labeled `enhancement`,
  and *some* linked issue must carry `ready-for-dev` — two INDEPENDENT checks over
  all linked issues. #18057 has `ready-for-dev` but not `enhancement`, and the App
  token cannot add labels upstream (403); solved by also linking parent PRD #9414
  (has `enhancement`, already cited by #18057). Local `--body-file` mode skips the
  API check. CI stays red only on the reserved `HUMAN:` note, which AI must not
  write. Workflow: repo is a shallow clone -> `git fetch --unshallow` before
  rebasing; after a rebase run `npm ci` + `npm run make-i18n` (gitignored
  `src/i18n/declaration.ts`) or typecheck fails. Details in
  [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 live-activity production evidence.** The review comment
  on `activity-view.tsx:102` ("needs real-backend evidence") belongs to **PR 18064**
  (feat(activity) live activity view), NOT 18067 (Bahasa i18n) — 18067 was a red
  herring. To run a real backend: SDK 1.53.0 in venv `/tmp/oh153venv` on port 60002
  (branch needs agent-server >=1.51.0), Vite on 12000, tunnel work-1. **Key gotcha:**
  a null `llm.api_key` makes LiteLLM return "Missing credentials" as a 500, which the
  SDK's auth-error detector ignores, so the managed-key refresh hook never fires.
  Set a dummy key (`sk-managed-placeholder`) to force a real 401 -> hook fetches the
  managed key -> retry succeeds. Evidence (2 PNGs + webm) committed to
  `.pr/live-activity-view-evidence/` on branch `feat/live-activity-view` (commit
  56a026d, pushed to fork `origin`); PR body + comment updated; `HUMAN:` block
  preserved byte-for-byte (CRLF). PR stays draft. Details in
  [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 review fixes pushed (`5e0097c`).** Fixed the
  `deriveSubagents` bug: a `TaskObservation` pairs to its action by `action_id`,
  which is the ACTION EVENT `id`, not the action's `tool_call_id`. Also added
  `mergeActivityTail` carry-forward, Load-more/empty-state split, runtime-URL
  cache scoping, and an `isPinnableRoute` fix for `/activity`. Verified
  `npm test` = 787 files / 8375 passed. Two workflow gotchas: (1) `npm test`
  takes ~10 min here and the pre-commit `lint-staged` hook exceeds the 30s
  terminal soft-timeout — run `git commit`/`push` and `npm test` in the
  background and poll a log; (2) the PR body is CRLF, so API edits must split on
  the `HUMAN:` block and preserve it byte-for-byte (AI must never edit it).
  Details in [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 activity-tail round-2 review fixes (`27f8977`).** Deeper
  findings on `feat/live-activity-view`: (1) completed delegations reopened after
  the bounded 60-event window trimmed their observation — fixed by retaining
  resolving `TaskObservation`s in a separate `resolvedObservations` buffer field
  and re-attaching them whenever the action is present (pruned to stay bounded);
  (2) a >600-event backlog re-read the same newest 20 pages forever — fixed by
  storing the incomplete range's `resumePageId` cursor and resuming it; (3) a
  failed cloud page was silently degraded to an empty page — fixed with
  `strictPagination:true` (first-page failure => unsupported-filter fallback,
  later-page failure => keep watermark+cursor and retry); (4) rotation detection
  lost on unmount / inactive-list transitions — moved to a map keyed by query
  identity, storing only a `sessionGeneration()` fingerprint (later changed to a
  keyed HMAC in round 3), never the credential. `npm test` = 788 files / 8394
  passed; lint clean. Note:
  `OpenHandsEvent` union exposes `action_id` only on `ObservationEvent` (guard with
  `isObservationEvent`). Details in [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 activity-tail round-3 review fixes (`66823ea`).** New
  findings: (1) `sessionGeneration` was an unkeyed FNV-1a fingerprint — replaced
  with a 128-bit HMAC-SHA-256 tag keyed by a random per-process secret (returns
  `null` without Web Crypto rather than a guessable hash); (2) a resumed backlog
  ending on an empty terminal page left the watermark unchanged so every poll
  replayed it — added `pendingHighWatermark` to `ActivityTailBuffer`, committed on
  range completion; (3) the module-scoped session-generation map grew for the
  page's lifetime — moved to `WeakMap<QueryClient, Map<...>>` with a query-cache
  `removed` subscription attached for the CLIENT's lifetime (gcTime evicts after
  unmount); (4) added a successive-poll retention test. TS gotcha:
  `crypto.subtle.importKey` needs `Uint8Array<ArrayBuffer>`, not bare
  `Uint8Array`. `npm test` = 788 files / 8399 passed. Details in
  [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 activity-tail round-4 review fixes (`c049595`).** Four
  session-identity findings. Core insight: React Query does **not** re-run
  `queryFn` when the query key is unchanged, so a key-rotation effect alone can
  never reset the tail promptly; correctness must live in the merge. Fix: drop
  the generation map entirely and stamp each cached `ActivityTailBuffer` with
  `sessionId = runtimeUrl#hmac(sessionKey)`; `queryFn` reuses `watermark`/
  `resumePageId`/`supportsTimestampFilter` only when `cached.sessionId` matches
  the current identity. This also fixes the pending-HMAC race (old effect
  recorded the generation only after its HMAC resolved, so a key change arriving
  first left no generation) and the non-secure-origin case (no `crypto.subtle`
  ⇒ identity `null` ⇒ unknown, never a match ⇒ tail not reused). Removed the
  test-only `__activitySessionGenerationCountForTests` export. Test gotchas: to
  fake the pending HMAC, `vi.spyOn(crypto.subtle, "sign")` returning a
  never-resolving promise for the first call; to test no-Web-Crypto,
  `Object.defineProperty(crypto, "subtle", { value: undefined })` and call
  `client.refetchQueries(...)` explicitly (a rerender alone does not refetch).
  `npm test` = 788 files / 8401 passed / 7 todo. Details in
  [2026-10-06.md](./2026-10-06.md).

- **2026-10-06 — PR #18064 activity-tail round-5 review fixes (`a86a581`).** Two
  follow-ups: (1) `queryFn` recognised a rotated credential but still passed
  `cached` to `mergeActivityTail`, so the old watermark + `supportsTimestampFilter`
  leaked into the new session and a runtime clock behind that watermark hid the
  new session's earlier events from later filtered polls forever — fixed with
  `const previous = sameSession ? cached : undefined` passed to the merge in all
  three branches; (2) a null `session_api_key` (same URL) skipped the reset
  guard, leaving a stamped tail on screen behind unauthenticated polls — the
  guard now clears on any mismatch but only for a stamped (`sessionId !==
  undefined`) tail, so an identity-less origin does not reset on every list
  update. Test gotchas: `client.setQueryData(key, data, { updatedAt: 0 })` is
  required to seed a *stale* entry that still fetches (`staleTime: 5s`); make a
  per-conversation mock answer key off `searchEvents.mock.calls[n][0]` (the
  conversation id) instead of `mockResolvedValueOnce` (order-fragile with
  parallel queries). Both new tests verified to fail pre-fix.
  `npm test` = 788 files / 8403 passed / 7 todo. Details in
  [2026-10-06.md](./2026-10-06.md).
