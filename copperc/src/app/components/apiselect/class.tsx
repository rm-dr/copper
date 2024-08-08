import { APIclient } from "@/app/_util/api";
import { ApiSelector } from "./api";
import { UseFormReturnType } from "@mantine/form";

async function update_classes(dataset: string | null) {
	if (dataset === null) {
		return Promise.resolve(null);
	}

	const { data, error } = await APIclient.GET("/class/list", {
		params: {
			query: {
				dataset,
			},
		},
	});

	if (error !== undefined) {
		throw error;
	}

	return data.map(({ name, handle }) => {
		return {
			label: name,
			value: handle.toString(),
			disabled: false,
		};
	});
}

export function ClassSelector(params: {
	onSelect: (value: number | null) => void;
	selectedDataset: string | null;
	form?: {
		form: UseFormReturnType<any>;
		key: string;
	};
}) {
	return (
		<ApiSelector
			onSelect={(v) => params.onSelect(v === null ? null : parseInt(v))}
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
			form={params.form}
		/>
	);
}
