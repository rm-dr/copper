// For use with ApiSelector
//
// TODO: make these components with icons

export async function update_datasets(_: null) {
	const res = await fetch("/api/dataset/list");
	const data: { name: string; ds_type: string }[] = await res.json();

	return data.map(({ name }) => {
		return {
			label: name,
			value: name,
			disabled: false,
		};
	});
}

export async function update_pipelines(dataset: string | null) {
	if (dataset === null) {
		return Promise.resolve(null);
	}

	const res = await fetch(
		"/api/pipeline/list?" +
			new URLSearchParams({
				dataset,
			}),
	);

	const data: { input_type: string; name: string }[] = await res.json();

	return data.map(({ name, input_type }) => {
		return {
			label: name,
			value: name,
			disabled: input_type == "None",
		};
	});
}

export async function update_classes(dataset: string | null) {
	if (dataset === null) {
		return Promise.resolve(null);
	}

	const res = await fetch(
		"/api/class/list?" +
			new URLSearchParams({
				dataset,
			}),
	);

	const data: { name: string }[] = await res.json();

	return data.map(({ name }) => {
		return {
			label: name,
			value: name,
			disabled: false,
		};
	});
}
