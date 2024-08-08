import { ApiSelector } from "./api";

async function update_pipelines(dataset: string | null) {
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

export function PipelineSelector(params: {
	onSelect: (value: string | null) => void;
	selectedDataset: string | null;
}) {
	return (
		<ApiSelector
			onSelect={params.onSelect}
			update_params={params.selectedDataset}
			update_list={update_pipelines}
			messages={{
				nothingmsg_normal: "No pipelines found",
				nothingmsg_empty: "This dataset has no pipelines",
				placeholder_error: "could not fetch pipelines",
				placeholder_normal: "select pipeline",
				message_null: "select a pipeline",
				message_loading: "fetching pipelines...",
			}}
		/>
	);
}
