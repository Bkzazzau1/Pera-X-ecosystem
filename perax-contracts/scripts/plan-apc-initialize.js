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
const apc = tokenomics.adaptivePriceControl;
const hash = crypto.createHash('sha256').update(canonical(policy.parameters)).digest('hex');
const policyReady = apc?.policyStatus === 'approved'
  && apc.policyVersion === policy.policyVersion
  && apc.policyHashSha256 === policy.policyHashSha256
  && hash === policy.policyHashSha256
  && canonical(apc.approvedParameters) === canonical(policy.parameters)
  && Array.isArray(apc.unresolvedNumericalPolicies)
  && apc.unresolvedNumericalPolicies.length === 0;
console.log(JSON.stringify({
  action: 'initialize_apc',
  policyReady,
  executionReady: false,
  policyVersion: policy.policyVersion,
  policyHashSha256: policy.policyHashSha256,
  parameters: policy.parameters,
  blockedBy: policyReady ? ['reviewed production addresses', 'successful full validation pipeline', 'independent security approval'] : ['exact APC Policy V1 approval and synchronization'],
}, null, 2));
if (!policyReady) process.exitCode = 1;
