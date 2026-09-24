function parse(tag) {
  const match = tag.match(
    /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/,
  );
  if (!match) throw new Error(`Invalid release tag: ${tag}`);
  return {
    core: [Number(match[1]), Number(match[2]), Number(match[3])],
    pre: match[4] ? match[4].split(".") : [],
  };
}

function compareIdentifier(a, b) {
  const aNumeric = /^\d+$/.test(a);
  const bNumeric = /^\d+$/.test(b);
  if (aNumeric && bNumeric) return Number(a) - Number(b);
  if (aNumeric) return -1;
  if (bNumeric) return 1;
  return a.localeCompare(b);
}

function compare(a, b) {
  for (let index = 0; index < 3; index += 1) {
    if (a.core[index] !== b.core[index]) return a.core[index] - b.core[index];
  }
  if (a.pre.length === 0 && b.pre.length === 0) return 0;
  if (a.pre.length === 0) return 1;
  if (b.pre.length === 0) return -1;
  const length = Math.max(a.pre.length, b.pre.length);
  for (let index = 0; index < length; index += 1) {
    if (a.pre[index] === undefined) return -1;
    if (b.pre[index] === undefined) return 1;
    const result = compareIdentifier(a.pre[index], b.pre[index]);
    if (result !== 0) return result;
  }
  return 0;
}

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}

const currentTag = args.get("--current");
const previousTag = args.get("--previous");
if (!currentTag || !previousTag) {
  throw new Error("Usage: check-release-order.mjs --current vX.Y.Z --previous vA.B.C");
}

if (compare(parse(currentTag), parse(previousTag)) <= 0) {
  throw new Error(
    `Release ${currentTag} must be newer than the latest published release ${previousTag}`,
  );
}

console.log(`Release order verified: ${previousTag} -> ${currentTag}`);
