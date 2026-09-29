"""Hand-audit verdicts for the three requirement tags still live in crates/economy/src.

Scope note, and the reason this file is three entries rather than eleven.
Eight of the eleven sites originally nominated (JouleAllocator/FR-CIV-ECON-003,
MarketState/FR-CIV-MARKET-002..005, and SCHEMA_VERSION/FR-CIV-ECON-001-MARKET
+ FR-CIV-MARKET-001) were already unbound earlier today, in commits 25fe9fcc and
8d719130. Their live tags were replaced by `// [unbound] <id>: <reason>` blocks,
and a re-run of the applier's upward tag-block walk over crates/economy/src finds
exactly the three ids adjudicated below plus the unrelated FR-ECON-005 and
FR-CIV-TEST-006 tags. Listing those eight again would ask the applier to delete
lines that are already documented as removed, so they are recorded here in KEEP
with the evidence that the removal landed, not in SITES.

Every id in the SITES dicts below is a false binding whose tag is still present in
source. Each reason quotes the authoritative SHALL sentence with its file:line.
"""

KEEP = {
    # FR-CIV-CURRENCY-TRUST is a TRUE binding on CurrencyTrust and is only listed
    # here so the applier does not abort the FR-CIV-MARKET-007 removal as
    # "unclassified". The rustdoc at currency_trust.rs:83 names it, so the
    # applier's upward walk pulls that line into the CurrencyTrust tag block.
    # It is genuinely implemented: step_currency_trust (currency_trust.rs:264)
    # drives trust from price/supply pressure with a hyperinflation penalty
    # (:315), and there are acceptance tests at :406 and :441.
    #
    # This must live in module-level KEEP rather than a per-site "__keep__"
    # entry. The applier reads the per-site form as `set(removed.get("__keep__",
    # ()))`, so a string becomes a set of single characters and a tuple of one
    # string is rejected by _peek.py, which calls .split() on every dict value.
    # KEEP is unioned into `keep` for every site, which is exactly the intent.
    "FR-CIV-CURRENCY-TRUST": (
        "true binding on CurrencyTrust; retained. Listed so the FR-CIV-MARKET-007 "
        "removal is not blocked as unclassified."
    ),

    # The following eight were already unbound before this pass; the removal
    # landed and is committed. Re-listing them in SITES would be a no-op at best
    # and would re-assert ids the applier is designed to never re-read (its
    # `is_tag_line` explicitly skips lines carrying the [unbound] marker).
    #
    # Evidence that the removal is real, not merely intended:
    #   git show 25fe9fcc -- crates/economy/src/lib.rs
    #     -// FR-CIV-ECON-001-MARKET   (deleted)
    #     -// FR-CIV-MARKET-001        (deleted)
    #     +// [unbound] FR-CIV-ECON-001-MARKET: MIS-BOUND. ...
    #     +// [unbound] FR-CIV-MARKET-001: MIS-BOUND. ...
    #   git show 8d719130 --stat   (crates/economy/src/{allocation.rs,market.rs})
    #   grep -rn '\[unbound\]' crates/economy/src  -> 6 ids, 3 files
    "FR-CIV-ECON-001-MARKET": (
        "already unbound at crates/economy/src/lib.rs:99 by commit 25fe9fcc. The "
        "premise that this removal 'did not stick' is wrong: the tag is gone from "
        "source and the [unbound] reason is committed. A stale dry-run reported "
        "'no tag block above' because the applier only counts lines that are live "
        "tags; every remaining line in that block carries [unbound], so the upward "
        "walk correctly finds no tag to remove. There is nothing to remove and no "
        "tool bug. The recorded reason is also substantively wrong and should be "
        "superseded -- see the SCHEMA_VERSION follow-up in the audit report."
    ),
    "FR-CIV-MARKET-001": (
        "already unbound at crates/economy/src/lib.rs:100 by commit 25fe9fcc. Same "
        "premise correction as FR-CIV-ECON-001-MARKET above. This binding was never "
        "true: SCHEMA_VERSION is a u32 wire marker and cannot be a condition vector."
    ),
    "FR-CIV-ECON-003": (
        "already unbound from JouleAllocator at crates/economy/src/allocation.rs:60 "
        "by commit 8d719130. JouleAllocator is a unit struct whose allocate() is a "
        "demand.min(budget) clamp; it selects no numeraire."
    ),
    "FR-CIV-MARKET-002": (
        "already unbound from MarketState at crates/economy/src/market.rs:44 by "
        "commit 8d719130. MarketState wraps BTreeMap<String,i64> of per-good prices; "
        "no projector function or price-field read-out exists in the crate."
    ),
    "FR-CIV-MARKET-003": (
        "already unbound from MarketState at crates/economy/src/market.rs:45 by "
        "commit 8d719130. MarketState has no soft-membership weight vector over "
        "market types; a good maps to one price, which is a hard switch."
    ),
    "FR-CIV-MARKET-004": (
        "already unbound from MarketState at crates/economy/src/market.rs:46 by "
        "commit 8d719130. No tatonnement iteration exists; `tatonnement` has zero "
        "hits across crates/."
    ),
    "FR-CIV-MARKET-005": (
        "already unbound from MarketState at crates/economy/src/market.rs:47 by "
        "commit 8d719130. MarketState holds no trust, volume, camera-proximity or "
        "activity field, and no locale-to-CDA upgrade path."
    ),
    "FR-CIV-MARKET-006": (
        "already unbound from AllocationRegime at crates/economy/src/allocation.rs:185 "
        "by commit 8d719130. The enum is a three-variant vocabulary that "
        "allocate_with dispatches on; no coercion signal ever flips a locale's regime."
    ),
}

SITES = [
    (
        "crates/economy/src/currency_trust.rs",
        "CurrencyTrust",
        {
            "FR-CIV-MARKET-007": (
                "The requirement in docs/design/polities-markets.md:144 is 'FR-CIV-MARKET-007 -- Money emerges, "
                "is not declared. No hardcoded currency. A numeraire emerges as the good with the highest "
                "liquidity (trade frequency x acceptability across counterparties) in a region; prices may "
                "re-denominate against it.' CurrencyTrust is a per-currency trust accumulator: it holds a "
                "trust score in basis points plus cumulative trade volume, gained/lost trust and pass "
                "counters (currency_trust.rs:90-111), and the doc comment above it at :83-87 describes a "
                "trust score, not a unit of account. It names a single issued currency by caller-supplied "
                "currency_id, which is the declared-currency model the requirement explicitly rejects. Nothing "
                "in it selects a good, ranks goods by liquidity, or re-denominates a price against a chosen "
                "numeraire. Grepping crates/ for 'numeraire' returns 8 hits and every one is a comment, an "
                "[unbound] reason, or a guard test asserting the numeraire read-out is still absent "
                "(crates/economy/tests/fr_civ_econ_cluster.rs:464-468); there is no numeraire type, no "
                "numeraire field, and no numeraire selection function anywhere in the workspace. The 'liquidity' "
                "hits in crates/economy/src/market.rs:659-798 are order-book depth accounting on a single "
                "good, not cross-good liquidity used to pick a unit of account. The nearest symbol to the "
                "requirement is the price book itself, MultiGoodMarket in crates/economy/src/market.rs:832, "
                "which stores per-good prices but likewise never selects a numeraire. The true implementing "
                "symbol for this requirement does not exist."
            ),
        },
        [
            "`CurrencyTrust` really does implement currency trust, tracked by the separate "
            "FR-CIV-CURRENCY-TRUST id. FR-CIV-MARKET-007 was bound to it only because the "
            "word 'currency' appeared in the name. The requirement is about an emerging unit "
            "of account, and the crate has no code that picks one.",
        ],
    ),
    (
        "crates/economy/src/institution.rs",
        "LedgerSide",
        {
            "FR-CIV-MARKET-008": (
                "The requirement in docs/design/polities-markets.md:146 is 'FR-CIV-MARKET-008 -- Credit/debt "
                "via institution postings. Deferred settlement reuses institution::InstitutionLedger "
                "double-entry postings: a credit market records a debt as a balanced posting (debtor liability "
                "<-> creditor asset) that settles later. Conservation (verify_conservation) holds throughout; "
                "default/forgiveness is a posting that writes the debt off and souring DiplomacyMatrix "
                "relationships.' The posting half of this is genuinely implemented: "
                "InstitutionLedger::post (crates/economy/src/institution.rs:220) writes a balanced "
                "debit/credit pair, InstitutionLedger::verify_conservation "
                "(crates/economy/src/institution.rs:323) checks it, and the self-trade branch at "
                "crates/economy/src/allocator.rs:444-453 records the debtor/creditor pair that the design doc "
                "names as the credit-market seed. But that behavior is the ledger's, not the enum's. "
                "LedgerSide is a two-variant enum (crates/economy/src/institution.rs:32-37) naming which "
                "account a leg posts to: Macro(AccountId) or Institution(InstitutionId). It has no methods, "
                "no fields, and no behavior, so it cannot record a debt, schedule a later settlement, hold a "
                "liability across ticks, or write off a default. The debt lifecycle the requirement demands "
                "is absent: grepping crates/ for 'deferred' returns only unrelated comment hits and finds no "
                "deferred-settlement type, and 'self_trade' has 0 hits. Critically, "
                "InstitutionLedger::post at :230-235 actively REJECTS the self-trade case the design doc "
                "cites as the credit seed, returning InstitutionLedgerError::SelfPosting. The allocator works "
                "around that rejection at allocator.rs:444-453 by posting the same account on both legs, so "
                "the obligation is logged but never created as a balance. Tag belongs on "
                "InstitutionLedger / InstitutionPosting, not on the leg-enum."
            ),
        },
        [
            "This is the closest call in the batch. The double-entry posting machinery the "
            "requirement leans on is real and is exercised from the engine through "
            "EconomyState::institutions. The error is placement: the tag sits on the enum "
            "that names an account, not on the type that records and settles a debt.",
        ],
    ),
    (
        "crates/economy/src/lib.rs",
        "LedgerEntry",
        {
            "FR-CIV-ECON-004": (
                "The requirement in agileplus-specs/civ-021-recovered-requirements/spec.md:76-78 is "
                "'FR-CIV-ECON-004 -- Policy-driven fiscal control via crates/engine/src/policy.rs.' "
                "LedgerEntry is a four-field bookkeeping row (crates/economy/src/lib.rs:115-124) holding "
                "tick, debit, credit and account, and its own doc comment at :112 calls it a 'stub; full "
                "double-entry pairs in CIV-0100'. It records a posting after the fact; it does not drive one. "
                "Policy-driven fiscal control means a policy object is evaluated each tick and its output "
                "changes fiscal behavior, and that is implemented in a different crate: the Policy trait at "
                "crates/engine/src/policy.rs:64 reads WorldState and returns ControlSignals, "
                "ControlSignals::tax_rates at crates/engine/src/policy.rs:53 carries per-institution tax "
                "rates in basis points, and Simulation::phase_policy (policy.rs:9-11) evaluates it each tick "
                "immediately before phase_economy. LedgerEntry has no connection to any of that: it is never "
                "passed a policy, never holds a rate, and never selects a tax. The only writers of the type "
                "are economy/src/lib.rs:170 and :331-343, all plain appends. Note also that the spec names "
                "crates/engine/src/policy.rs as the required artifact, so the spec never pointed at this "
                "struct at all. Tag comes off; the fiscal control path is policy.rs."
            ),
        },
        [
            "A ledger row cannot be a fiscal policy. The spec for this id names a different "
            "file, which is the strongest single piece of evidence that the binding was "
            "mis-placed rather than merely under-tested.",
        ],
    ),
]
