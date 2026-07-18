from pathlib import Path
import re

ROOT = Path.cwd()
CONTEXTS = ROOT / "perax-contracts/programs/perax-core/src/contexts.rs"
CARGO_TOML = ROOT / "perax-contracts/programs/perax-core/Cargo.toml"
PACKAGE_JSON = ROOT / "perax-contracts/package.json"
WORKFLOW = ROOT / ".github/workflows/perax-contracts-ci.yml"


def replace_exact(path: Path, old: str, new: str, expected: int = 1) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != expected:
        raise SystemExit(
            f"{path}: expected {expected} matches for {old!r}, found {count}"
        )
    path.write_text(text.replace(old, new))


# Box every Anchor Account wrapper in the context module. This reduces account
# deserialization stack usage without changing addresses, constraints, or IDL
# account ordering.
contexts = CONTEXTS.read_text()
if "Box<Account<'info," in contexts:
    raise SystemExit("contexts.rs already contains boxed accounts")
pattern = re.compile(
    r"(?P<prefix>\bpub\s+[A-Za-z_][A-Za-z0-9_]*:\s*)"
    r"Account<'info,\s*(?P<kind>[A-Za-z_][A-Za-z0-9_:]*)>(?P<suffix>,)"
)
contexts, boxed_count = pattern.subn(
    lambda match: (
        f"{match.group('prefix')}Box<Account<'info, {match.group('kind')}>>"
        f"{match.group('suffix')}"
    ),
    contexts,
)
if boxed_count < 120:
    raise SystemExit(f"unexpectedly low global boxed-account count: {boxed_count}")
CONTEXTS.write_text(contexts)
print(f"boxed {boxed_count} Anchor account fields across all instruction contexts")

# Anchor 0.31 moves each init constraint into its own closure, substantially
# reducing try_accounts stack use. 0.31.1 also fixes proc-macro IDL failures.
replace_exact(CARGO_TOML, 'anchor-lang = "0.30.1"', 'anchor-lang = "0.31.1"')
replace_exact(CARGO_TOML, 'anchor-spl = "0.30.1"', 'anchor-spl = "0.31.1"')
replace_exact(PACKAGE_JSON, '"@coral-xyz/anchor": "^0.30.1"', '"@coral-xyz/anchor": "^0.31.1"')

workflow = WORKFLOW.read_text()
changes = [
    (
        "Install Cargo toolchain for Anchor metadata",
        "Install Rust toolchains for Anchor and IDL generation",
        1,
    ),
    (
        "rustup toolchain install 1.85.0 --profile minimal",
        """rustup toolchain install 1.89.0 --profile minimal
          rustup toolchain install nightly-2025-04-14 --profile minimal""",
        1,
    ),
    ("agave-3.1.14", "agave-2.1.0", 2),
    ("https://release.anza.xyz/v3.1.14/install", "https://release.anza.xyz/v2.1.0/install", 1),
    ("cargo +1.79.0 install \\", "cargo +1.89.0 install \\", 1),
    ("--tag v0.30.1", "--tag v0.31.1", 1),
    (
        "RUSTUP_TOOLCHAIN=1.85.0 anchor build 2>&1 | tee /tmp/anchor-build.log",
        """RUSTUP_TOOLCHAIN=nightly-2025-04-14 anchor build 2>&1 | tee /tmp/anchor-build.log
          if grep -E 'Stack offset of [0-9]+ exceeded max offset of 4096|Stack frame size of [0-9]+ exceeded max allowed size of 4096|overwrites values in the frame' /tmp/anchor-build.log; then
            echo 'Unsafe SBF stack frame detected.'
            exit 1
          fi""",
        1,
    ),
    (
        "RUSTUP_TOOLCHAIN=1.85.0 anchor test --provider.cluster localnet",
        "anchor test --skip-build --provider.cluster localnet",
        1,
    ),
]
for old, new, expected in changes:
    count = workflow.count(old)
    if count != expected:
        raise SystemExit(
            f"workflow expected {expected} matches for {old!r}, found {count}"
        )
    workflow = workflow.replace(old, new)
WORKFLOW.write_text(workflow)
print("upgraded Anchor Rust/TypeScript/CLI to 0.31.1 with the recommended Agave 2.1 toolchain")
