import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}

const version = args.get("--version");
const channel = (args.get("--channel") ?? "stable").toLowerCase();

if (!version) {
  throw new Error("Usage: npm run release:prepare -- --version X.Y.Z --channel stable|beta");
}
if (!["stable", "beta"].includes(channel)) {
  throw new Error("Release channel must be stable or beta.");
}

const stablePattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const betaPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)-beta\.(0|[1-9]\d*)$/;
if (channel === "stable" && !stablePattern.test(version)) {
  throw new Error("Stable releases require X.Y.Z with no prerelease suffix.");
}
if (channel === "beta" && !betaPattern.test(version)) {
  throw new Error("Beta releases require X.Y.Z-beta.N.");
}

function parse(value) {
  const match = value.match(/^(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?$/);
  if (!match) throw new Error("Unsupported Byte version format: " + value);
  return { core: [Number(match[1]), Number(match[2]), Number(match[3])], beta: match[4] === undefined ? null : Number(match[4]) };
}

function compare(left, right) {
  for (let index = 0; index < 3; index += 1) {
    if (left.core[index] !== right.core[index]) return left.core[index] - right.core[index];
  }
  if (left.beta === null && right.beta === null) return 0;
  if (left.beta === null) return 1;
  if (right.beta === null) return -1;
  return left.beta - right.beta;
}

const packagePath = path.join(root, "package.json");
const lockPath = path.join(root, "package-lock.json");
const tauriPath = path.join(root, "src-tauri", "tauri.conf.json");
const cargoPath = path.join(root, "src-tauri", "Cargo.toml");
const changelogPath = path.join(root, "CHANGELOG.md");

const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
const packageLock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
const tauri = JSON.parse(fs.readFileSync(tauriPath, "utf8"));
let cargo = fs.readFileSync(cargoPath, "utf8");
let changelog = fs.readFileSync(changelogPath, "utf8");

const current = tauri.version;
if (compare(parse(version), parse(current)) <= 0) {
  throw new Error("Prepared version " + version + " must be newer than current version " + current + ".");
}

const unreleasedMatch = changelog.match(/## \[Unreleased\]\r?\n([\s\S]*?)(?=\r?\n## \[|$)/);
if (!unreleasedMatch) {
  throw new Error("CHANGELOG.md must contain an ## [Unreleased] section.");
}
const unreleasedBody = unreleasedMatch[1].trim();
if (!unreleasedBody) {
  throw new Error("CHANGELOG.md [Unreleased] must contain release notes before preparation.");
}

const date = new Date().toISOString().slice(0, 10);
const replacement = "## [Unreleased]\n\n## [" + version + "] - " + date + "\n\n" + unreleasedBody + "\n";
changelog = changelog.replace(unreleasedMatch[0], replacement);

packageJson.version = version;
packageLock.version = version;
if (packageLock.packages?.[""]) packageLock.packages[""].version = version;
tauri.version = version;

const cargoVersion = /(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m;
if (!cargoVersion.test(cargo)) {
  throw new Error("Could not locate [package] version in src-tauri/Cargo.toml.");
}
cargo = cargo.replace(cargoVersion, "$1" + version + "$2");

fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2) + "\n");
fs.writeFileSync(lockPath, JSON.stringify(packageLock, null, 2) + "\n");
fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + "\n");
fs.writeFileSync(cargoPath, cargo);
fs.writeFileSync(changelogPath, changelog);

console.log("Prepared Byte " + version + " (" + channel + ").");
console.log("Review CHANGELOG.md, commit the version changes, and run the full PR certification before tagging.");
