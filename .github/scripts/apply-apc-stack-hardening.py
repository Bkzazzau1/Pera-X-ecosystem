from pathlib import Path
import re

ROOT = Path.cwd()
CONTEXTS = ROOT / "perax-contracts/programs/perax-core/src/contexts.rs"
WORKFLOW = ROOT / ".github/workflows/perax-contracts-ci.yml"

APC_CONTEXTS = [
    "InitializeRecoveryPool",
    "InitializeApc",
    "SubmitApcObservation",
    "ActivateNextApcBand",
    "ExecuteApcRelease",
    "DepositCounterweightProceeds",
    "RecordDeferredBurn",
    "ExecuteDeferredBurn",
    "ConfirmApcAbsorption",
    "EnterApcRecovery",
    "ExecuteCounterweightPurchase",
    "RecoverySwapAdapter",
    "PauseApc",
]


def box_context_accounts(text: str, struct_name: str) -> tuple[str, int]:
    start_marker = f"pub struct {struct_name}<'info> {{"
    start = text.find(start_marker)
    if start == -1:
        raise SystemExit(f"missing Anchor context {struct_name}")

    body_start = start + len(start_marker)
    end = text.find("\n}\n", body_start)
    if end == -1:
        raise SystemExit(f"unterminated Anchor context {struct_name}")

    block = text[start:end + 3]
    if "Box<Account<'info," in block:
        raise SystemExit(f"{struct_name} already contains boxed accounts")

    pattern = re.compile(
        r"(?P<prefix>\n\s*pub\s+[A-Za-z_][A-Za-z0-9_]*:\s*)"
        r"Account<'info,\s*(?P<kind>[A-Za-z_][A-Za-z0-9_:]*)>(?P<suffix>,)"
    )
    boxed, count = pattern.subn(
        lambda match: (
            f"{match.group('prefix')}Box<Account<'info, {match.group('kind')}>>"
            f"{match.group('suffix')}"
        ),
        block,
    )
    if count == 0:
        raise SystemExit(f"{struct_name} had no Account fields to box")

    return text[:start] + boxed + text[end + 3:], count


contexts = CONTEXTS.read_text()
total = 0
for context in APC_CONTEXTS:
    contexts, count = box_context_accounts(contexts, context)
    total += count
    print(f"boxed {count:2d} account fields in {context}")

if total < 45:
    raise SystemExit(f"unexpectedly low boxed-account count: {total}")
CONTEXTS.write_text(contexts)
print(f"boxed {total} APC account fields in total")

workflow = WORKFLOW.read_text()
replacements = [
    (
        "Install Cargo toolchain for Anchor metadata",
        "Install Rust toolchain for Anchor and IDL generation",
    ),
    (
        "rustup toolchain install 1.85.0 --profile minimal",
        "rustup toolchain install 1.89.0 --profile minimal",
    ),
    (
        "RUSTUP_TOOLCHAIN=1.85.0 anchor build 2>&1 | tee /tmp/anchor-build.log",
        """RUSTUP_TOOLCHAIN=1.89.0 anchor build 2>&1 | tee /tmp/anchor-build.log
          if grep -E 'Stack offset of [0-9]+ exceeded max offset of 4096|Stack frame size of [0-9]+ exceeded max allowed size of 4096' /tmp/anchor-build.log; then
            echo 'Unsafe SBF stack frame detected.'
            exit 1
          fi""",
    ),
    (
        "RUSTUP_TOOLCHAIN=1.85.0 anchor test --provider.cluster localnet",
        "RUSTUP_TOOLCHAIN=1.89.0 anchor test --provider.cluster localnet",
    ),
]
for old, new in replacements:
    count = workflow.count(old)
    if count != 1:
        raise SystemExit(f"workflow expected one match for {old!r}, found {count}")
    workflow = workflow.replace(old, new)
WORKFLOW.write_text(workflow)
print("aligned Anchor host tooling to Rust 1.89 and enabled stack-frame enforcement")
