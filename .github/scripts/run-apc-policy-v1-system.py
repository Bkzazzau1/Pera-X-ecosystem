from pathlib import Path
import runpy

source = Path('.github/scripts/apply-apc-policy-v1-system.py')
target = Path('/tmp/apply-apc-policy-v1-system-fixed.py')
text = source.read_text()
old = '''  const expression = match[1].replaceAll('_', '').trim();
  if (expression.includes('* PEX_DECIMALS')) return BigInt(expression.split('*')[0].trim());
  if (expression.includes('* APC_USDC_BASE_UNITS')) return BigInt(expression.split('*')[0].trim());
  return BigInt(expression);'''
new = '''  const rawExpression = match[1].trim();
  if (rawExpression.includes('* PEX_DECIMALS')) return BigInt(rawExpression.split('*')[0].replaceAll('_', '').trim());
  if (rawExpression.includes('* APC_USDC_BASE_UNITS')) return BigInt(rawExpression.split('*')[0].replaceAll('_', '').trim());
  return BigInt(rawExpression.replaceAll('_', ''));'''
if text.count(old) != 1:
    raise SystemExit('expected one Rust-number parser block')
text = text.replace(old, new)
target.write_text(text)
runpy.run_path(str(target), run_name='__main__')
