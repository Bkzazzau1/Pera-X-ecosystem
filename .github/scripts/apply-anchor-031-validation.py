from pathlib import Path
import json

ROOT = Path.cwd()
CONTRACTS = ROOT / "perax-contracts"


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old!r}")
    path.write_text(text.replace(old, new))


cargo = CONTRACTS / "programs/perax-core/Cargo.toml"
replace_once(cargo, 'rust-version = "1.79"', 'rust-version = "1.85"')
replace_once(cargo, 'anchor-lang = "0.30.1"', 'anchor-lang = "0.31.1"')
replace_once(cargo, 'anchor-spl = "0.30.1"', 'anchor-spl = "0.31.1"')

package_path = CONTRACTS / "package.json"
package = json.loads(package_path.read_text())
package["dependencies"]["@coral-xyz/anchor"] = "0.31.1"
package["scripts"]["check:build-sizes"] = "node scripts/check-build-sizes.js"
package_path.write_text(json.dumps(package, indent=2) + "\n")

anchor_toml = CONTRACTS / "Anchor.toml"
text = anchor_toml.read_text()
if "[toolchain]" not in text:
    text = text.rstrip() + '\n\n[toolchain]\nanchor_version = "0.31.1"\nsolana_version = "2.1.0"\npackage_manager = "npm"\n'
else:
    raise SystemExit("Anchor.toml already has a toolchain section; review before replacing")
anchor_toml.write_text(text)

check_workflow = ROOT / ".github/workflows/perax-contracts-check.yml"
text = check_workflow.read_text().replace("1.79.0", "1.85.0").replace("rust-1.79.0", "rust-1.85.0")
check_workflow.write_text(text)

size_script = CONTRACTS / "scripts/check-build-sizes.js"
size_script.write_text(r'''const fs = require('fs');
const path = require('path');
const root = path.resolve(__dirname, '..');
const programPath = path.join(root, 'target/deploy/perax_core.so');
const idlPath = path.join(root, 'target/idl/perax_core.json');
if (!fs.existsSync(programPath)) throw new Error('Missing built program target/deploy/perax_core.so');
if (!fs.existsSync(idlPath)) throw new Error('Missing generated IDL target/idl/perax_core.json');
const programBytes = fs.statSync(programPath).size;
const idlBytes = fs.statSync(idlPath).size;
const maximumProgramBytes = 10 * 1024 * 1024;
const maximumIdlBytes = 2 * 1024 * 1024;
if (programBytes <= 0 || programBytes > maximumProgramBytes) throw new Error(`Program size ${programBytes} is outside the reviewed limit.`);
if (idlBytes <= 0 || idlBytes > maximumIdlBytes) throw new Error(`IDL size ${idlBytes} is outside the reviewed limit.`);
const report = { programBytes, maximumProgramBytes, idlBytes, maximumIdlBytes };
fs.writeFileSync('/tmp/perax-build-sizes.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
''')

rust_tests = CONTRACTS / "programs/perax-core/src/tests.rs"
marker = "apc_account_size_report_and_limits"
text = rust_tests.read_text()
if marker in text:
    raise SystemExit("account size test already exists")
text += r'''

// apc_account_size_report_and_limits
#[test]
fn apc_account_size_report_and_limits() {
    const MAX_REVIEWED_ACCOUNT_BYTES: usize = 10 * 1024 * 1024;
    let sizes = [
        ("ApcConfig", 8 + ApcConfig::INIT_SPACE),
        ("ApcState", 8 + ApcState::INIT_SPACE),
        ("ApcObservation", 8 + ApcObservation::INIT_SPACE),
        ("ApcBandRecord", 8 + ApcBandRecord::INIT_SPACE),
        ("ApcReleaseRecord", 8 + ApcReleaseRecord::INIT_SPACE),
        ("CounterweightConfig", 8 + CounterweightConfig::INIT_SPACE),
        ("CounterweightDepositRecord", 8 + CounterweightDepositRecord::INIT_SPACE),
        ("DeferredBurnRecord", 8 + DeferredBurnRecord::INIT_SPACE),
        ("ApcRecoveryRecord", 8 + ApcRecoveryRecord::INIT_SPACE),
        ("RecoveryPoolConfig", 8 + RecoveryPoolConfig::INIT_SPACE),
    ];
    for (name, bytes) in sizes {
        println!("{name}: {bytes} bytes");
        assert!(bytes > 8 && bytes <= MAX_REVIEWED_ACCOUNT_BYTES);
    }
}
'''
rust_tests.write_text(text)

full_workflow = ROOT / ".github/workflows/perax-contracts-ci.yml"
full_workflow.write_text(r'''name: Pera-X Contracts CI

on:
  push:
    branches: [main]
    paths:
      - "perax-contracts/**"
      - "perax-market-engine/**"
      - ".github/workflows/perax-contracts-ci.yml"
  pull_request:
    branches: [main]
    paths:
      - "perax-contracts/**"
      - "perax-market-engine/**"
      - ".github/workflows/perax-contracts-ci.yml"
  workflow_dispatch:

jobs:
  contracts:
    name: Build, inspect and transaction-test Anchor program
    runs-on: ubuntu-latest
    timeout-minutes: 60
    defaults:
      run:
        working-directory: perax-contracts
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      - name: Reject tracked local key material
        shell: bash
        run: |
          tracked_secrets="$(git -C .. ls-files | grep -E '(^|/)\.local/|(^|/)(id|keypair|wallet|solana-wallet)\.json$|\.keypair\.json$' || true)"
          if [ -n "$tracked_secrets" ]; then echo "$tracked_secrets"; exit 1; fi
      - name: Set up Rust 1.85
        uses: dtolnay/rust-toolchain@1.85.0
        with:
          components: rustfmt
      - name: Set up Node.js
        uses: actions/setup-node@v6
        with:
          node-version: "24"
          package-manager-cache: false
      - name: Cache Cargo, Anchor and build output
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            ~/.avm
            perax-contracts/target
          key: ${{ runner.os }}-anchor-0.31.1-agave-2.1.0-rust-1.85-${{ hashFiles('perax-contracts/Cargo.lock', 'perax-contracts/package-lock.json') }}
      - name: Install Agave 2.1.0
        shell: bash
        run: |
          sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.0/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> "$GITHUB_PATH"
      - name: Install Anchor CLI 0.31.1
        shell: bash
        run: |
          if [ ! -x "$HOME/.avm/bin/anchor-0.31.1" ]; then
            cargo install --git https://github.com/solana-foundation/anchor avm --locked --force
            avm install 0.31.1
          fi
          mkdir -p "$HOME/.local/bin"
          ln -sf "$HOME/.avm/bin/anchor-0.31.1" "$HOME/.local/bin/anchor"
          echo "$HOME/.local/bin" >> "$GITHUB_PATH"
      - name: Tool versions
        shell: bash
        run: |
          rustc --version
          cargo --version
          solana --version
          anchor --version
          node --version
      - name: Create ephemeral CI wallet
        shell: bash
        run: |
          mkdir -p .local
          solana-keygen new --no-bip39-passphrase --silent --force --outfile .local/devnet-deployer.json
      - name: Install JavaScript dependencies
        run: npm ci
      - name: Validate canonical policy and planning gates
        shell: bash
        run: |
          npm run simulate:apc-policy-v1
          npm run validate:tokenomics
          npm run plan:allocation
          npm run plan:mint
          npm run plan:initialize
          npm run plan:apc
          node scripts/initialize-apc-program.js
          node scripts/plan-fixed-supply-finalization.js
          node scripts/plan-allocation-execution.js
          node scripts/check-devnet-readiness.js
          node scripts/check-anchor-deploy-readiness.js
      - name: Rust formatting, unit and property tests
        shell: bash
        run: |
          cargo fmt --all -- --check
          cargo test --locked --all-targets
          cargo check --locked --all-targets
      - name: TypeScript and market-engine tests
        shell: bash
        run: |
          npm run typecheck
          cd ../perax-market-engine
          npm ci
          npm run typecheck
          npm test
      - name: Build Anchor program and inspect stack frames
        shell: bash
        run: |
          set -o pipefail
          anchor build 2>&1 | tee /tmp/anchor-build.log
          if grep -E 'Stack offset of [0-9]+ exceeded max offset of 4096|Stack frame size of [0-9]+ exceeded max allowed size of 4096|overwrites values in the frame' /tmp/anchor-build.log; then
            echo 'Unsafe SBF stack frame detected.'
            exit 1
          fi
      - name: Compare deterministic IDL
        shell: bash
        run: |
          cp target/idl/perax_core.json /tmp/perax_core.first.json
          anchor build >/tmp/anchor-build-second.log 2>&1
          cmp /tmp/perax_core.first.json target/idl/perax_core.json
          if [ -f idl/perax_core.json ]; then cmp idl/perax_core.json target/idl/perax_core.json; fi
      - name: Program and account size checks
        shell: bash
        run: |
          npm run check:build-sizes
          cargo test --locked apc_account_size_report_and_limits -- --nocapture | tee /tmp/apc-account-sizes.log
      - name: Run local-validator transaction tests
        shell: bash
        run: anchor test --skip-build --provider.cluster localnet 2>&1 | tee /tmp/anchor-local-validator.log
      - name: Upload validation evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: perax-apc-policy-v1-validation-evidence
          path: |
            /tmp/anchor-build.log
            /tmp/anchor-build-second.log
            /tmp/anchor-local-validator.log
            /tmp/perax-build-sizes.json
            /tmp/apc-account-sizes.log
            perax-contracts/target/idl/perax_core.json
            perax-contracts/target/deploy/perax_core.so
          if-no-files-found: warn
''')

print("Prepared Anchor 0.31.1 / Agave 2.1.0 validation upgrade")
