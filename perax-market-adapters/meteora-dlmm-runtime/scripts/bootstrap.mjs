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
await writeFile(resolve(packageRoot, "package-lock.json"), lock);

await verify(
  resolve(packageRoot, "src/index.ts"),
  "dae5d76cb66093baac20a68caad040d7fb118877a716cd36986cca42ab8965b1",
  "Meteora runtime source",
);
await verify(
  resolve(packageRoot, "package-lock.json"),
  "68e280dba4394c61d7cfcd522c03c2099cbad39d61c4d67e9e6724dab4b6409d",
  "Meteora dependency lock",
);
console.log("Verified Meteora runtime source and deterministic dependency lock.");

async function verify(path, expected, label) {
  const digest = createHash("sha256").update(await readFile(path)).digest("hex");
  if (digest !== expected) {
    throw new Error(`${label} checksum ${digest} does not match ${expected}`);
  }
}
