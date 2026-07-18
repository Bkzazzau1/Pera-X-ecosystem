# APC Numerical Policy Version 1 — Simulation Report

- Policy hash: `17f93bacb0cfa5346a466258117908068f1f0cd67054f8b61c7d40818dfe84bb`
- Deterministic seed: `20260718`
- Candidate configurations evaluated: **2,916**
- Candidates satisfying every hard safety constraint: **480**
- Deterministic market scenarios: **1,920**
- Randomized invariant cases: **25,000**
- Maximum APC-added immediate price impact: **498 bps**
- Minimum counterweight coverage ratio: **1.365×**
- Wallet-splitting invariance: **passed** for 1, 2, 5 and 20 wallets

## Approval result

APC Policy Version 1 is the top-ranked candidate under the documented governance score after every candidate first passes the hard impact, coverage, monotonic-risk, release-budget and reserve-floor constraints. The simulation is conservative: it treats the active liquidity as a constant-product approximation and assumes released PEX may be sold into the same active liquidity.

## Limits of this proof

This simulation proves deterministic policy consistency and economic invariants for the stated model. It does not replace the Anchor local-validator suite, program stack inspection, IDL comparison, or an independent security and economic audit. No deployment decision may rely on this report alone.
