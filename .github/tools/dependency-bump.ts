import { join } from "https://deno.land/std@0.224.0/path/join.ts";
import { exists } from "https://deno.land/std@0.224.0/fs/exists.ts";

const [path, dependency, version] = Deno.args;
const ghOutput = Deno.env.get("GITHUB_OUTPUT")!;
if (!ghOutput) {
	throw new Error(
		"Job must be run in a GitHub Actions environment with GITHUB_OUTPUT set.",
	);
}

if (!path || !dependency || !version) {
	throw new Error(
		"Invalid arguments. Usage: deno run -A .github/tools/dependency-bump.ts <path> <dependency> <version|patch|minor|major>",
	);
}

console.info(
	`Bumping version in '${path}' for dependency '${dependency}' to '${version}'`,
);
const cargoTomlPath = path.endsWith("Cargo.toml")
	? path
	: join(path, "Cargo.toml");

if (!await exists(cargoTomlPath)) {
	throw new Error(`Cargo.toml not found at ${cargoTomlPath}`);
}
let cargoToml = await Deno.readTextFile(cargoTomlPath);

if (["patch", "minor", "major"].includes(version)) {
	const versionRegex = new RegExp(
		`(${dependency}\\s*=\\s*{[^}]*version\\s*=\\s*")([^"]+)(")`,
	);
	const match = cargoToml.match(versionRegex);
	if (!match) {
		throw new Error(
			`Dependency ${dependency} with version not found in ${cargoTomlPath}`,
		);
	}
	let [major, minor, patch] = match[2].split(".").map(Number);

	switch (version) {
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
	cargoToml = cargoToml.replace(
		versionRegex,
		`$1${newVersion}$3`,
	);
} else {
	cargoToml = cargoToml.replace(
		new RegExp(`(${dependency}\\s*=\\s*{[^}]*version\\s*=\\s*")([^"]+)(")`),
		`$1${version}$3`,
	).replace(
		new RegExp(`(${dependency}\\s*=\\s*")([^"]+)(")`),
		`$1${version}$3`,
	);
}

await Deno.writeTextFile(cargoTomlPath, cargoToml);
