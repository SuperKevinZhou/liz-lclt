import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const rootDir = process.cwd();
const rawVersion = process.argv[2];

if (!rawVersion) {
  console.error("Usage: node scripts/sync-version.mjs <version-or-tag>");
  process.exit(1);
}

const version = rawVersion.startsWith("v") ? rawVersion.slice(1) : rawVersion;

if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`Invalid semantic version: ${rawVersion}`);
  process.exit(1);
}

const updates = [
  {
    file: "package.json",
    replacer: (content) =>
      content.replace(
        /("version"\s*:\s*")(\d+\.\d+\.\d+)(")/,
        `$1${version}$3`,
      ),
  },
  {
    file: "package-lock.json",
    replacer: (content) =>
      content
        .replace(/("version"\s*:\s*")(\d+\.\d+\.\d+)(")/, `$1${version}$3`)
        .replace(
          /(""\s*:\s*\{\s*"name"\s*:\s*"liz-lclt",\s*"version"\s*:\s*")(\d+\.\d+\.\d+)(")/,
          `$1${version}$3`,
        ),
  },
  {
    file: path.join("src-tauri", "Cargo.toml"),
    replacer: (content) =>
      content.replace(
        /(name = "liz-lclt"\r?\nversion = ")(\d+\.\d+\.\d+)(")/,
        `$1${version}$3`,
      ),
  },
  {
    file: path.join("src-tauri", "Cargo.lock"),
    replacer: (content) =>
      content.replace(
        /(name = "liz-lclt"\r?\nversion = ")(\d+\.\d+\.\d+)(")/,
        `$1${version}$3`,
      ),
  },
  {
    file: path.join("src-tauri", "tauri.conf.json"),
    replacer: (content) =>
      content.replace(
        /("version"\s*:\s*")(\d+\.\d+\.\d+)(")/,
        `$1${version}$3`,
      ),
  },
];

for (const update of updates) {
  const filePath = path.join(rootDir, update.file);
  const original = await readFile(filePath, "utf8");
  const next = update.replacer(original);

  if (next === original) {
    console.log(`${update.file} already uses ${version}`);
    continue;
  }

  await writeFile(filePath, next);
  console.log(`Updated ${update.file} -> ${version}`);
}
