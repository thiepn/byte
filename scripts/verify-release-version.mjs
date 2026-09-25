import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const packageLock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const tauri = JSON.parse(
  fs.readFileSync(path.join(root, "src-tauri", "tauri.conf.json"), "utf8"),
);
const cargo = fs.readFileSync(path.join(root, "src-tauri", "Cargo.toml"), "utf8");
const cargoLock = fs.readFileSync(path.join(root, "src-tauri", "Cargo.lock"), "utf8");

const cargoPackage = cargo.match(/\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
if (!cargoPackage) {
  throw new Error("Could not read [package] version from src-tauri/Cargo.toml");
}

const cargoLockPackage = cargoLock.match(
  /\[\[package\]\]\s*\nname\s*=\s*"byte-desktop"\s*\nversion\s*=\s*"([^"]+)"/,
);
if (!cargoLockPackage) {
  throw new Error("Could not read byte-desktop version from src-tauri/Cargo.lock");
}

const versions = {
  packageJson: packageJson.version,
  packageLock: packageLock.version,
  packageLockRoot: packageLock.packages?.[""]?.version,
  tauri: tauri.version,
  cargo: cargoPackage[1],
  cargoLock: cargoLockPackage[1],
};

const unique = new Set(Object.values(versions));
if (unique.size !== 1) {
  throw new Error(
    "Release versions must match across package.json, package-lock.json, tauri.conf.json, Cargo.toml, and Cargo.lock: " +
      JSON.stringify(versions),
  );
}

const version = versions.tauri;
const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
if (!semver.test(version)) {
  throw new Error(`Release version is not valid SemVer: ${version}`);
}

const requestedTagIndex = process.argv.indexOf("--tag");
const requestedTag =
  requestedTagIndex >= 0 ? process.argv[requestedTagIndex + 1] : undefined;

if (requestedTag) {
  const expectedTag = `v${version}`;
  if (requestedTag !== expectedTag) {
    throw new Error(
      `Release tag must exactly match the app version: expected ${expectedTag}, received ${requestedTag}`,
    );
  }
}

const configuredWindows = tauri.app?.windows ?? [];
const expectedWindowLabels = ["companion", "main", "quick-panel"];
const actualWindowLabels = configuredWindows.map((window) => window.label).sort();

if (JSON.stringify(actualWindowLabels) !== JSON.stringify(expectedWindowLabels)) {
  throw new Error(
    "Byte must keep exactly the companion, main, and quick-panel configured windows",
  );
}
if (configuredWindows.some((window) => window.create !== false)) {
  throw new Error(
    "Every configured Byte window must use create:false so AppState is managed before frontend IPC starts",
  );
}

const bundle = tauri.bundle ?? {};
const windows = bundle.windows ?? {};
const nsis = windows.nsis ?? {};

if (tauri.productName !== "Byte" || tauri.mainBinaryName !== "Byte") {
  throw new Error("Production productName and mainBinaryName must both remain Byte");
}
if (tauri.identifier !== "io.github.thiepn.byte") {
  throw new Error("Production bundle identifier changed unexpectedly");
}
if (bundle.active !== true || !Array.isArray(bundle.targets) || !bundle.targets.includes("nsis")) {
  throw new Error("Production bundling must be active and include the NSIS target");
}
if (windows.allowDowngrades !== false) {
  throw new Error("Windows release policy must block downgrades");
}
if (nsis.installMode !== "currentUser") {
  throw new Error("Byte's NSIS installer must remain current-user scoped");
}
if (nsis.installerHooks !== "windows/hooks.nsh") {
  throw new Error("Byte's uninstall cleanup hook is not configured");
}

console.log(
  `Byte release configuration verified: v${version} / NSIS current-user / downgrades blocked`,
);
