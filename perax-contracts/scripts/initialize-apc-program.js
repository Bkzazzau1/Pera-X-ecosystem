const fs = require("fs");
const path = require("path");
const config = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../config/pex-tokenomics.json"), "utf8"));
if (!process.argv.includes("--execute")) {
  console.log("Dry run only. Use plan-apc-initialize.js to review the initialization gate.");
  process.exit(0);
}
if (config.adaptivePriceControl?.policyStatus !== "approved") {
  throw new Error("APC initialization blocked: numerical policy is not formally approved.");
}
throw new Error("Execution intentionally postponed until reviewed addresses and approved numerical values are supplied.");
