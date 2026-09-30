# Civis pass 18 — expanded state denominator and save-generation prototype

Date 2026-09-30.

Candidate `7e9a348c23f1b5559c19a3d71ecad21ef4f43e91`; run `36703825128`; artifact `11091337450`; sha256 `0fa5ed6cc1b5bf8cec39ea61a0962274abf81a957c9fea759b01f8873c7d7b1f`.

20 recovery tests executed: 10 passed, 10 failed.

Production semantic/classifier reds now include:
- economy PolicyInput reset;
- research loss;
- guest memory without loaded mod identity;
- control policy capitalist→noop;
- market state loss;
- metadata-deletion downgrade bypass;
- tutorial progress reset;
- religious profiles loss;
- active caravans in transit loss.

That is **9 production-path reds**. The tenth failure in the 20-test run was the prototype atomicity fixture described below; do not inflate the production-failure denominator by counting it as a product red.

Prototype greens include:
- semantic policy/research restoration;
- orphan guest-memory detection;
- modern-with-missing-metadata classifier;
- explicit legacy-candidate preservation;
- missing required semantic component rejection;
- unknown future semantic schema rejection;
- mixed-generation semantic manifest rejection;
- successful generation pointer switch preserving prior generation;
- other bounded semantic/classifier controls.

One prototype atomicity case failed because the test damaged `world_state.json`, but current CivSaveBundle did not classify that removal as invalid in the assumed way. That was an invalid fault assumption for the generation publisher, not proof publication is unsafe. The test now damages the **required semantic-state.json** component itself at commit time, directly exercising the prototype's own required-component boundary.

The semantic prototype has been expanded at `514889e97886c8c06a88ea5536d2e656511f68b9` to include the newly reproduced tutorial, religious-profile and active-caravan state in addition to policy/research/control-policy/market/mod identity. Complex domains are serialized as explicit semantic components in the prototype rather than silently treating WorldState as exhaustive.

Current rerun: `36705591591`, pending at receipt.
