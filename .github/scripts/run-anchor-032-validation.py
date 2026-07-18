from pathlib import Path
import runpy

source = Path('.github/scripts/apply-anchor-031-validation.py')
target = Path('/tmp/apply-anchor-032-validation.py')
text = source.read_text()

text = text.replace('0.31.1', '0.32.1')
text = text.replace('2.1.0', '2.3.0')

old_install = '''      - name: Install Anchor CLI 0.32.1
        shell: bash
        run: |
          if [ ! -x "$HOME/.avm/bin/anchor-0.32.1" ]; then
            cargo install --git https://github.com/solana-foundation/anchor avm --locked --force
            avm install 0.32.1
          fi
          mkdir -p "$HOME/.local/bin"
          ln -sf "$HOME/.avm/bin/anchor-0.32.1" "$HOME/.local/bin/anchor"
          echo "$HOME/.local/bin" >> "$GITHUB_PATH"
'''
new_install = '''      - name: Install Anchor native build dependencies
        shell: bash
        run: |
          sudo apt-get update
          sudo apt-get install -y pkg-config libssl-dev libudev-dev build-essential
      - name: Install Rust 1.89 for Anchor CLI compilation
        shell: bash
        run: rustup toolchain install 1.89.0 --profile minimal
      - name: Install Anchor CLI 0.32.1
        shell: bash
        run: |
          cargo +1.89.0 install \\
            --git https://github.com/solana-foundation/anchor \\
            --tag v0.32.1 \\
            anchor-cli \\
            --locked \\
            --force
'''
if text.count(old_install) != 1:
    raise SystemExit('expected one permanent workflow Anchor install block')
text = text.replace(old_install, new_install)
text = text.replace('~/.avm\n', '~/.cargo/bin\n')

for required in [
    'anchor-lang = "0.32.1"',
    'anchor_version = "0.32.1"',
    'solana_version = "2.3.0"',
    'https://release.anza.xyz/v2.3.0/install',
    '--tag v0.32.1',
    'libudev-dev',
]:
    if required not in text:
        raise SystemExit(f'missing transformed requirement: {required}')

target.write_text(text)
runpy.run_path(str(target), run_name='__main__')
