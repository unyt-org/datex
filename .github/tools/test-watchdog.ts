const CRATE_NAME = "datex-core";

type TestCase = {
	kind: "test";
	name: string;
	ignored: boolean;
};

async function getTests(
	features: string[] = [],
	allFeatures = false,
): Promise<TestCase[]> {
	const cmd = new Deno.Command("cargo", {
		args: [
			"nextest",
			"list",
			...(features.length ? ["--features", ...features] : []),
			...(allFeatures ? ["--all-features"] : []),
			"--lib",
			"--message-format",
			"json",
		],
		env: {
			RUSTFLAGS: "-Awarnings",
		},
		stderr: "inherit",
	});

	const { stdout, success } = await cmd.output();
	if (!success) {
		throw new Error("Failed to list tests");
	}
	const cases = JSON.parse(
		new TextDecoder().decode(stdout),
	)["rust-suites"][CRATE_NAME]["testcases"] as { [key: string]: TestCase };
	return Object.entries(cases)
		.filter(([_, testCase]) => !testCase.ignored)
		.map(([name, data]) => {
			return {
				...data,
				name,
			};
		});
}

const allTests = await getTests([]);
console.log(`Found ${allTests.length} tests`);

const stdTests = await getTests(["decompiler", "compiler", "target_native"]);
console.log(`Found ${stdTests.length} std tests`);

const nostdTests = await getTests([
	"decompiler",
	"compiler",
	"allow_unsigned_blocks",
]);
console.log(`Found ${nostdTests.length} nostd tests`);
