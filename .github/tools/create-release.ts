import { join } from "https://deno.land/std@0.224.0/path/join.ts";
import { exists } from "https://deno.land/std@0.224.0/fs/exists.ts";

const [path, type] = Deno.args;

const ghOutput = Deno.env.get("GITHUB_OUTPUT")!;
if (!ghOutput) {
    throw new Error(
        "Job must be run in a GitHub Actions environment with GITHUB_OUTPUT set.",
    );
}

if (!["major", "minor", "patch"].includes(type)) {
    throw new Error(
        "Invalid version bump type. Use 'major', 'minor', or 'patch'.",
    );
}

console.info(
    `Bumping version in '${path}' with type ${type}...`,
);

const cargoTomlPath = path.endsWith("Cargo.toml")
    ? path
    : join(path, "Cargo.toml");

if (!await exists(cargoTomlPath)) {
    throw new Error(`Cargo.toml not found at ${cargoTomlPath}`);
}
const cargoToml = await Deno.readTextFile(cargoTomlPath);

// Extract version
const versionRegex = /version\s*=\s*"(\d+)\.(\d+)\.(\d+)"/;
const match = versionRegex.exec(cargoToml);
if (!match) {
    throw new Error("Version not found in Cargo.toml");
}

let [major, minor, patch] = match.slice(1).map(Number);

switch (type) {
    case "major":
        major++;
        minor = 0;
        patch = 0;
        break;
    case "minor":
        minor++;
        patch = 0;
        break;
    case "patch":
        patch++;
        break;
}

const newVersion = `${major}.${minor}.${patch}`;
const updatedCargoToml = cargoToml.replace(
    versionRegex,
    `version = "${newVersion}"`,
);
await Deno.writeTextFile(cargoTomlPath, updatedCargoToml);
await Deno.writeTextFile(ghOutput, `NEW_VERSION=${newVersion}`, {
    append: true,
});

console.info(`Version updated to ${newVersion}`);
