const required = ["ANCHOR_PROVIDER_URL", "PERAX_CORE_PROGRAM_ID", "PEX_MINT_ADDRESS"];
const missing = required.filter((name) => !process.env[name]);
if (missing.length) {
  console.log(`APC state verification not executed. Missing: ${missing.join(", ")}`);
  process.exit(0);
}
console.log("APC verification inputs are present. On-chain verification remains a post-deployment operation.");
