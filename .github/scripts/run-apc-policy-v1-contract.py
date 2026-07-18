from pathlib import Path
import runpy

source = Path('.github/scripts/apply-apc-policy-v1-contract.py')
target = Path('/tmp/apply-apc-policy-v1-contract-fixed.py')
text = source.read_text()
text = text.replace(
    '    if count != 1:\n        raise SystemExit(f"{path}: expected one match, found {count}: {old[:100]!r}")\n    path.write_text(text.replace(old, new))',
    '    if count < 1:\n        raise SystemExit(f"{path}: expected at least one match, found {count}: {old[:100]!r}")\n    path.write_text(text.replace(old, new, 1))',
)
text = text.replace(
    '        assert!(resumption <= config.deferred_burn_window_cap || config.deferred_burn_window_cap < resumption);',
    '        let executable = resumption.min(config.deferred_burn_window_cap);\n        assert!(executable <= resumption);\n        assert!(executable <= config.deferred_burn_window_cap);',
)
text += r'''

# Update existing tests for the exact Policy V1 classifications and support-price arguments.
tests = SRC / "tests.rs"
replace_once(
    tests,
    "    assert_eq!(tier, 2);\n    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 1_500);\n",
    "    assert_eq!(tier, 3);\n    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 750);\n",
)
replace_once(
    tests,
    "    let mut apc = test_apc_state(config.state);\n    let amount = 100 * PEX_DECIMALS;\n",
    "    let mut apc = test_apc_state(config.state);\n    apc.deferred_burn_amount = 1_000 * PEX_DECIMALS;\n    let amount = 100 * PEX_DECIMALS;\n",
)
for old, new in [
    (
        "        &config, &apc, 100_000, 1_000_000, 1_000_000, 10_000,\n",
        "        &config, &apc, 100_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,\n",
    ),
    (
        "        &config, &apc, 300_000, 1_000_000, 1_000_000, 10_000,\n",
        "        &config, &apc, 300_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,\n",
    ),
    (
        "        &config, &apc, 100_000, 1_000_000, 1_000_000, 10_000,\n",
        "        &config, &apc, 100_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,\n",
    ),
    (
        "        1_000_000,\n        10_000 + config.recovery_cooldown_seconds - 1,\n",
        "        1_000_000,\n        2_000,\n        10_000,\n        10_000 + config.recovery_cooldown_seconds - 1,\n",
    ),
]:
    replace_once(tests, old, new)
'''
if text == source.read_text():
    raise SystemExit('contract runner did not alter the guarded transformation')
target.write_text(text)
runpy.run_path(str(target), run_name='__main__')
