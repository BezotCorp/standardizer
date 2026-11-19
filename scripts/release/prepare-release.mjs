import fs from "node:fs";
import { spawnSync } from "node:child_process";
import semanticRelease from "semantic-release";

const outputFile = process.env.GITHUB_OUTPUT;

function setOutput(name, value) {
  if (outputFile) {
    fs.appendFileSync(outputFile, `${name}=${value}\n`);
  } else {
    console.log(`${name}=${value}`);
  }
}

const result = await semanticRelease({
  branches: ["main"],
  tagFormat: "v${version}",
  dryRun: true,
  ci: false,
  plugins: [
    [
      "@semantic-release/commit-analyzer",
      {
        preset: "conventionalcommits"
      }
    ],
    [
      "@semantic-release/release-notes-generator",
      {
        preset: "conventionalcommits"
      }
    ]
  ]
});

if (!result) {
  setOutput("created", "false");
  process.exit(0);
}

const version = result.nextRelease.version;
const tag = `v${version}`;
const notes = result.nextRelease.notes.trim();

const cargo = spawnSync(
  "cargo",
  ["set-version", "--workspace", version],
  {
    stdio: "inherit",
    shell: false
  }
);

if (cargo.status !== 0) {
  throw new Error(`cargo set-version ${version} failed`);
}

const changelogPath = "CHANGELOG.md";
const date = new Date().toISOString().slice(0, 10);
const section = `## [${version}] - ${date}\n\n${notes}\n\n`;

let changelog = fs.existsSync(changelogPath)
  ? fs.readFileSync(changelogPath, "utf8")
  : "# Changelog\n\n";

const versionHeading = `## [${version}]`;

if (!changelog.includes(versionHeading)) {
  if (changelog.startsWith("# Changelog")) {
    const firstLineEnd = changelog.indexOf("\n");

    changelog =
      `${changelog.slice(0, firstLineEnd + 1)}\n` +
      section +
      changelog.slice(firstLineEnd + 1).replace(/^\n+/, "");
  } else {
    changelog = `# Changelog\n\n${section}${changelog}`;
  }

  fs.writeFileSync(changelogPath, changelog);
}

setOutput("created", "true");
setOutput("version", version);
setOutput("tag", tag);

console.log(`Prepared ${tag}`);
