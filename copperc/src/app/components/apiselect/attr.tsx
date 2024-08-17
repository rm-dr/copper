import { APIclient } from "@/app/_util/api";
import { ApiSelector } from "./api";

async function update_attrs(params: {
	dataset: string | null;
	class: number | null;
}) {
	if (params.dataset === null || params.class === null) {
		return Promise.resolve(null);
	}

	// this is a bit inefficient, but I guess that's fine.
	let { data, error } = await APIclient.GET("/class/list", {
		params: { query: { dataset: params.dataset } },
	});

	if (data === undefined) {
		throw error;
	}

	const c = data.find((x) => x.handle == params.class);
	if (c === undefined) {
		return Promise.resolve(null);
	}

	return c.attrs.map(({ name, handle }) => ({
		label: name,
		value: handle.toString(),
		disabled: false,
	}));
}

export function AttrSelector(params: {
	onSelect: (value: number | null) => void;
	selectedDataset: string | null;
	selectedClass: number | null;
}) {
	return (
		<ApiSelector
			onSelect={(v) => params.onSelect(v === null ? null : parseInt(v))}
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
