/**
 * E2E serve chain: production build, then vite preview on 127.0.0.1:4173.
 * Used by Playwright webServer (build+preview only — never dev/HMR).
 */
import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWin = process.platform === "win32";
const npmCmd = isWin ? "npm.cmd" : "npm";

function run(cmd, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      cwd: appRoot,
      stdio: "inherit",
      shell: isWin,
      env: process.env,
    });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${cmd} ${args.join(" ")} exited ${code}`));
      }
    });
  });
}

await run(npmCmd, ["run", "build"]);

const preview = spawn(
  npmCmd,
  ["run", "preview", "--", "--host", "127.0.0.1", "--port", "4173", "--strictPort"],
  {
    cwd: appRoot,
    stdio: "inherit",
    shell: isWin,
    env: process.env,
  },
);

preview.on("error", (err) => {
  console.error(err);
  process.exit(1);
});

const shutdown = () => {
  if (!preview.killed) {
    preview.kill("SIGTERM");
  }
};
process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
process.on("exit", shutdown);

preview.on("exit", (code) => {
  process.exit(code ?? 1);
});
