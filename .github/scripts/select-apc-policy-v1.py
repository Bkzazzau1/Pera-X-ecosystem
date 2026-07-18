from pathlib import Path

root = Path.cwd()
simulator = root / "perax-contracts/scripts/simulate-apc-policy-v1.js"
text = simulator.read_text()

replacements = [
    (
        "for (const hourlyReleaseCapPex of ['2000000', '3000000'])",
        "for (const hourlyReleaseCapPex of ['2000000', '2500000', '3000000'])",
    ),
    (
        "Math.abs(normalThree - 3_000_000) / 3_000_000",
        "Math.abs(normalThree - 2_500_000) / 2_500_000",
    ),
    (
        "Math.abs(Number(candidate.hourlyReleaseCapPex) - 3_000_000) / 3_000_000",
        "Math.abs(Number(candidate.hourlyReleaseCapPex) - 2_500_000) / 2_500_000",
    ),
    (
        "if (totalBandBudget(candidate, 0, 3) < 3_000_000) continue;",
        "if (totalBandBudget(candidate, 0, 3) < 2_500_000) continue;",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one simulator match for {old!r}, found {count}")
    text = text.replace(old, new)

simulator.write_text(text)
print("updated APC candidate grid and governance score for the simulation-selected 2.5M PEX hourly cap")
