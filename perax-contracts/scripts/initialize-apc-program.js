const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const tokenomics = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/pex-tokenomics.json'), 'utf8'));
const policy = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/apc-policy-v1.json'), 'utf8'));
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
const hash = crypto.createHash('sha256').update(canonical(policy.parameters)).digest('hex');
const apc = tokenomics.adaptivePriceControl;
const exact = apc?.policyStatus === 'approved'
  && apc.policyVersion === policy.policyVersion
  && apc.policyHashSha256 === hash
  && hash === policy.policyHashSha256
  && canonical(apc.approvedParameters) === canonical(policy.parameters)
  && apc.unresolvedNumericalPolicies?.length === 0;
if (!exact) throw new Error('APC initialization blocked: Policy V1 is not exact across canonical JSON and tokenomics.');
if (!process.argv.includes('--execute')) {
  console.log(`Dry run only. APC Policy V${policy.policyVersion} (${policy.policyHashSha256}) is exact, but execution remains blocked.`);
  process.exit(0);
}
throw new Error('APC initialization intentionally blocked until reviewed production addresses, full CI/local-validator proof, and independent security approval are supplied.');
