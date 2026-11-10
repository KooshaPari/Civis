# Game process health check

`scripts/game-e2e.sh` can start the client and server and check process survival and report log errors without failing on them. With capture disabled, its automated clicks are approximate and failures are ignored; a passing result does not establish that the game reached Playing, the simulation advanced, or a save was restored.

The current Game E2E workflow disables frame capture, capture uploads, visual regression, and baseline updates. The workflow requires the client and server build dependencies and a display environment.

A gameplay acceptance witness still needs to observe the actual world and player action, verify state changes and save/load continuity, reject failed or stale observations, and clean up only processes it owns. Process survival and absence of logged errors are not that witness.
