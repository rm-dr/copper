import { ApiSelector } from "./api";

async function update_datasets(_: null) {
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

export function DatasetSelector(params: {
	onSelect: (value: string | null) => void;
}) {
	return (
		<ApiSelector
			onSelect={params.onSelect}
			update_params={null}
			update_list={update_datasets}
			messages={{
				nothingmsg_normal: "No datasets found",
				nothingmsg_empty: "No datasets are available",
				placeholder_error: "could not fetch datasets",
				placeholder_normal: "select dataset",
				message_loading: "fetching datasets...",
			}}
		/>
	);
}
