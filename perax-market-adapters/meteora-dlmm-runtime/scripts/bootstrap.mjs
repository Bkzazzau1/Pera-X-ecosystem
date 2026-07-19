import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = resolve(packageRoot, "../..");
const generatedRoot = resolve(packageRoot, "generated");

const python = spawnSync(
  "python3",
  [resolve(repositoryRoot, ".github/scripts/install-meteora-runtime.py")],
  { cwd: repositoryRoot, stdio: "inherit" },
);
if (python.status !== 0) {
  throw new Error(`Meteora runtime source generation failed with status ${python.status}`);
}

const parts = await Promise.all([
  readFile(resolve(generatedRoot, "package-lock.gz.b64.001"), "utf8"),
  readFile(resolve(generatedRoot, "package-lock.gz.b64.002"), "utf8"),
]);
const lock = gunzipSync(Buffer.from(parts.join(""), "base64"));
validateLock(JSON.parse(lock.toString("utf8")));
await writeFile(resolve(packageRoot, "package-lock.json"), lock);

await verify(
  resolve(packageRoot, "src/index.ts"),
  "dae5d76cb66093baac20a68caad040d7fb118877a716cd36986cca42ab8965b1",
  "Meteora runtime source",
);
console.log("Verified Meteora runtime source and deterministic dependency lock.");

function validateLock(value) {
  const root = value?.packages?.[""];
  if (
    value?.lockfileVersion !== 3 ||
    root?.name !== "perax-meteora-dlmm-runtime" ||
    root?.dependencies?.["@coral-xyz/anchor"] !== "0.31.0" ||
    root?.dependencies?.["@meteora-ag/dlmm"] !== "1.9.13" ||
    root?.dependencies?.["@solana/web3.js"] !== "^1.95.3" ||
    root?.dependencies?.["bigint-buffer"] !== "file:vendor/bigint-buffer-safe" ||
    root?.dependencies?.["decimal.js"] !== undefined
  ) {
    throw new Error("Meteora dependency lock does not match the approved runtime package");
  }
}

async function verify(path, expected, label) {
  const digest = createHash("sha256").update(await readFile(path)).digest("hex");
  if (digest !== expected) {
    throw new Error(`${label} checksum ${digest} does not match ${expected}`);
  }
}
