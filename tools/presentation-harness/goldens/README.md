# Technical screenshot baselines

Scenario goldens are pending the committed Scenario Runner replay fixture. Do not approve or commit viewport scenario baselines from the inline synthetic preflight.

When the verified fixture is present, run `npm test -- --update-snapshots` in the same Windows host, Edge channel, Playwright version, and device scale used for comparisons. Goldens are technical UI baselines only; they are not formal art approval. Keep any actual-vs-expected diff images with the corresponding run evidence under `artifacts/acceptance/pivot-presentation-harness-01/`.
