from pathlib import Path

path = Path("perax-contracts/tests/perax-core.ts")
text = path.read_text()
old_bigint = "    expect(pexAfterRecovery.amount).to.be.greaterThan(pexBeforeRecovery.amount);"
new_bigint = "    expect(pexAfterRecovery.amount > pexBeforeRecovery.amount).to.equal(true);"
old_observation = "    const wrongPoolObservation = observationPda(wrongPoolObservationId);\n    await expectFailure(() =>"
new_observation = "    const wrongPoolObservation = observationPda(wrongPoolObservationId);\n    const wrongPoolObservedAt = await currentChainTime();\n    await expectFailure(() =>"
old_await = "          observedAt: new anchor.BN(await currentChainTime()),"
new_await = "          observedAt: new anchor.BN(wrongPoolObservedAt),"
for old, new in ((old_bigint, new_bigint), (old_observation, new_observation), (old_await, new_await)):
    if text.count(old) != 1:
        raise SystemExit(f"unexpected TypeScript source state: {old}")
    text = text.replace(old, new)
path.write_text(text)
