import { ApiSelector } from "./api";

async function update_attrs(params: {
	dataset: string | null;
	class: string | null;
}) {
	if (params.dataset === null || params.class === null) {
		return Promise.resolve(null);
	}

	// TODO: list attrs endpoint, or take attrs as input?
	// this is a bit inefficient.
	const res = await fetch(
		"/api/class/list?" +
			new URLSearchParams({
				dataset: params.dataset,
			}),
	);

	const data: {
		name: string;
		attrs: {
			data_type: {};
			name: string;
		}[];
	}[] = await res.json();

	const c = data.find((x) => x.name == params.class);
	if (c === undefined) {
		return Promise.resolve(null);
	}

	return c.attrs.map(({ name }) => ({
		label: name,
		value: name,
		disabled: false,
	}));
}

export function AttrSelector(params: {
	onSelect: (value: string | null) => void;
	selectedDataset: string | null;
	selectedClass: string | null;
}) {
	return (
		<ApiSelector
			onSelect={params.onSelect}
			update_params={{
				dataset: params.selectedDataset,
				class: params.selectedClass,
			}}
			update_list={update_attrs}
			messages={{
				nothingmsg_normal: "No attributes found",
				nothingmsg_empty: "This class has no attributes",
				placeholder_error: "could not fetch attributes",
				placeholder_normal: "select an attribute",
				message_null: "select an attribute",
				message_loading: "fetching attributes...",
			}}
		/>
	);
}
