import { join } from "https://deno.land/std@0.224.0/path/join.ts";
import { exists } from "https://deno.land/std@0.224.0/fs/exists.ts";
import { walk } from "https://deno.land/std@0.224.0/fs/walk.ts";

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
const oldVersion = `${match[1]}.${match[2]}.${match[3]}`;

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

// Extract the crate name before modifying the file
const crateNameMatch = /\[package\][\s\S]*?name\s*=\s*"([^"]+)"/.exec(cargoToml);

const updatedCargoToml = cargoToml.replace(
    versionRegex,
    `version = "${newVersion}"`,
);
await Deno.writeTextFile(cargoTomlPath, updatedCargoToml);
await Deno.writeTextFile(ghOutput, `NEW_VERSION=${newVersion}`, {
    append: true,
});

console.info(`Version updated to ${newVersion}`);

// If the crate name was found, update dependency references in sibling crates
if (crateNameMatch) {
    const crateName = crateNameMatch[1];
    // Dependency names use hyphens; package names may use underscores
    const depName = crateName.replace(/_/g, "-");
    // Matches: dep-name = { ..., version = "old", ... } (possibly spanning lines)
    const depVersionRegex = new RegExp(
        `(${depName.replace(/-/g, "[-_]")}\\s*=\\s*\\{[^}]*version\\s*=\\s*")[^"]+`,
        "s",
    );
    for await (const entry of walk(".", {
        match: [/Cargo\.toml$/],
        skip: [/[/\\](target|node_modules|\.git)[/\\]/],
    })) {
        if (entry.path === cargoTomlPath) continue;
        const content = await Deno.readTextFile(entry.path);
        const updated = content.replace(depVersionRegex, `$1${newVersion}`);
        if (updated !== content) {
            await Deno.writeTextFile(entry.path, updated);
            console.info(
                `Updated ${depName} dependency in ${entry.path}: ${oldVersion} → ${newVersion}`,
            );
        }
    }
}
