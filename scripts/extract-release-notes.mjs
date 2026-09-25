import fs from "node:fs";

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}
const version = args.get("--version");
const output = args.get("--output");
if (!version || !output) {
  throw new Error("Usage: extract-release-notes.mjs --version X.Y.Z --output path");
}

const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
const escaped = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const marker = new RegExp("^## \\[" + escaped + "\\](?: - \\d{4}-\\d{2}-\\d{2})?\\s*$", "m");
const match = marker.exec(changelog);
if (!match) {
  throw new Error("CHANGELOG.md has no heading for version " + version + ".");
}

const rest = changelog.slice(match.index + match[0].length).replace(/^\r?\n/, "");
const nextHeading = rest.search(/^## \[/m);
const body = (nextHeading >= 0 ? rest.slice(0, nextHeading) : rest).trim();
if (!body) {
  throw new Error("CHANGELOG.md has no release notes for version " + version + ".");
}

fs.writeFileSync(output, body + "\n");
console.log("Extracted release notes for Byte " + version + " to " + output + ".");
