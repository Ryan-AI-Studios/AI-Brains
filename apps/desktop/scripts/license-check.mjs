// Production license gate for apps/desktop (S23).
// Uses license-checker-rseidelsohn; fails on GPL or AGPL licenses.
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const appRoot = join(__dirname, "..");
const require = createRequire(import.meta.url);

const checkerPkg = require.resolve("license-checker-rseidelsohn/package.json", {
  paths: [appRoot],
});
const checkerBin = join(dirname(checkerPkg), "bin", "license-checker-rseidelsohn.js");

// Human-readable summary first.
execFileSync(process.execPath, [checkerBin, "--production", "--summary"], {
  cwd: appRoot,
  stdio: "inherit",
});

// Machine-readable JSON for GPL/AGPL fail.
const jsonOut = execFileSync(
  process.execPath,
  [checkerBin, "--production", "--json"],
  {
    cwd: appRoot,
    encoding: "utf8",
  },
);

/** @type {Record<string, { licenses?: string | string[] }>} */
const packages = JSON.parse(jsonOut);
const forbidden = [];

for (const [name, info] of Object.entries(packages)) {
  const raw = info.licenses ?? "";
  const licenseText = Array.isArray(raw) ? raw.join(" OR ") : String(raw);
  // Product policy: zero AGPL/GPL in production tree.
  if (/\bAGPL\b/i.test(licenseText) || /\bGPL\b/i.test(licenseText)) {
    forbidden.push({ name, licenses: licenseText });
  }
}

if (forbidden.length > 0) {
  console.error("\nlicense:check FAILED — GPL/AGPL found in production tree:");
  for (const item of forbidden) {
    console.error(`  - ${item.name}: ${item.licenses}`);
  }
  process.exit(1);
}

console.log("\nlicense:check OK — no GPL/AGPL in production dependencies.");
