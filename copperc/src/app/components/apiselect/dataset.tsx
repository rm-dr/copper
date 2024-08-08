import { APIclient } from "@/app/_util/api";
import { ApiSelector } from "./api";

async function update_datasets(_: null) {
	const { data, error } = await APIclient.GET("/dataset/list");
	if (error !== undefined) {
		throw error;
	}

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
