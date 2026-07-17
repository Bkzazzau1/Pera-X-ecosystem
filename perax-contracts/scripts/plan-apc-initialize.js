const fs = require("fs");
const path = require("path");
const config = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../config/pex-tokenomics.json"), "utf8"));
const apc = config.adaptivePriceControl;
if (!apc) throw new Error("adaptivePriceControl policy is missing");
console.log(JSON.stringify({
  action: "initialize_apc",
  executable: apc.policyStatus === "approved",
  policyStatus: apc.policyStatus,
  firstActivationPriceScaled: apc.firstActivationPriceScaled,
  unresolvedNumericalPolicies: apc.unresolvedNumericalPolicies,
}, null, 2));
if (apc.policyStatus !== "approved") {
  console.log("BLOCKED: formal APC numerical approval is required before initialization.");
}
