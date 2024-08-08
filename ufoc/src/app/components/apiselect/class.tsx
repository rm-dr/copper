import { ApiSelector } from "./api";

async function update_classes(dataset: string | null) {
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

export function ClassSelector(params: {
	onSelect: (value: string | null) => void;
	selectedDataset: string | null;
}) {
	return (
		<ApiSelector
			onSelect={params.onSelect}
			update_params={params.selectedDataset}
			update_list={update_classes}
			messages={{
				nothingmsg_normal: "No classes found",
				nothingmsg_empty: "This dataset has no classes",
				placeholder_error: "could not fetch classes",
				placeholder_normal: "select a class",
				message_null: "select a class",
				message_loading: "fetching classes...",
			}}
		/>
	);
}
