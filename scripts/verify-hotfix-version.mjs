function parse(tag) {
  const match = tag.match(/^v?(\d+)\.(\d+)\.(\d+)$/);
  if (!match) throw new Error("Hotfix tags must be stable X.Y.Z versions: " + tag);
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}
const baseTag = args.get("--base");
const currentTag = args.get("--current");
if (!baseTag || !currentTag) {
  throw new Error("Usage: verify-hotfix-version.mjs --base vX.Y.Z --current vX.Y.Z");
}

const base = parse(baseTag);
const current = parse(currentTag);
if (base[0] !== current[0] || base[1] !== current[1]) {
  throw new Error("A hotfix must stay on the same major.minor line as its base release.");
}
if (current[2] <= base[2]) {
  throw new Error("A hotfix patch version must be greater than its base release.");
}

console.log("Hotfix version verified: " + baseTag + " -> " + currentTag + ".");
