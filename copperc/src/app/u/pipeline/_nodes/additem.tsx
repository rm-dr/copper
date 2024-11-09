import { Select } from "@mantine/core";
import {
	Node,
	NodeProps,
	useReactFlow,
	useUpdateNodeInternals,
} from "@xyflow/react";
import { BaseNode } from "./base";
import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { NodeDef, PipelineDataType } from ".";
import { useCallback } from "react";

type AddItemNodeType = Node<
	{
		dataset: null | number;
		class: null | number;

		inputs: {
			type: PipelineDataType;
			id: string;
			tooltip: string;
		}[];
	},
	"additem"
>;

function AddItemNodeElement({ data, id }: NodeProps<AddItemNodeType>) {
	const updateNodeInternals = useUpdateNodeInternals();
	const { deleteElements, getEdges, updateNodeData } = useReactFlow();

	const list = useQuery({
		queryKey: ["dataset/list"],

		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/list");
			if (res.response.status === 401) {
				location.replace("/");
			}

			updateNodeData(id, {
				...data,
				inputs:
					res.data === undefined || data.dataset === null || data.class === null
						? undefined
						: res.data
								.find((x) => x.id === data.dataset)
								?.classes.find((x) => x.id === data.class)
								?.attributes.map((x) => ({
									id: x.name,
									type: x.data_type.type,
									tooltip: x.name,
								})) || undefined,
			});

			return res.data;
		},
	});

	const updateInputs = useCallback(
		(old_data: typeof data, dataset: number | null, cls: number | null) => {
			updateNodeData(id, {
				...old_data,
				dataset,
				class: cls,
				inputs:
					list.data === undefined || dataset === null || cls === null
						? undefined
						: list.data
								.find((x) => x.id === dataset)
								?.classes.find((x) => x.id === cls)
								?.attributes.map((x) => ({
									id: x.name,
									type: x.data_type.type,
									tooltip: x.name,
								})) || undefined,
			});
		},
		[id, updateNodeData, list],
	);

	return (
		<>
			<BaseNode
				id={id}
				title={"Add Item"}
				inputs={data.inputs}
				outputs={
					data.class === null
						? []
						: [
								{
									id: "newid",
									tooltip: "Reference to new item",
									type: `Reference(${data.class})`,
								},
							]
				}
			>
				<Select
					clearable
					label="Select dataset"
					disabled={list.data === undefined}
					error={data.dataset !== null ? undefined : "No dataset selected"}
					data={
						list.data === undefined
							? []
							: list.data.map((x) => ({
									label: x.name,
									value: x.id.toString(),
								}))
					}
					onChange={(value) => {
						if (value === null) {
							updateInputs(data, null, null);
						} else {
							try {
								updateInputs(data, parseInt(value), null);
							} catch {}
						}

						updateNodeInternals(id);
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={data.dataset?.toString() || null}
				/>

				<Select
					clearable
					key={
						/* Make sure we get the right class list for the selected dataset */
						`class-${data.dataset}`
					}
					label="Select class"
					disabled={list.data === undefined || data.dataset === null}
					error={data.class !== null ? undefined : "No class selected"}
					data={
						list.data === undefined || data.dataset === null
							? []
							: list.data
									.find((x) => x.id === data.dataset)
									?.classes.map((x) => ({
										label: x.name,
										value: x.id.toString(),
									}))
					}
					onChange={(value) => {
						if (value === null) {
							updateInputs(data, data.dataset, null);
						} else {
							try {
								updateInputs(data, data.dataset, parseInt(value));
							} catch {}
						}

						updateNodeInternals(id);
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={data.class?.toString() || null}
				/>
			</BaseNode>
		</>
	);
}

export const AddItemNode: NodeDef<AddItemNodeType> = {
	xyflow_node_type: "additem",
	copper_node_type: "AddItem",
	node: AddItemNodeElement,

	initialData: {
		dataset: null,
		class: null,
		inputs: [],
	},

	getInputs: (data) =>
		data.inputs.map((x) => {
			return {
				id: x.id,
				type: x.type,
			};
		}),

	getOutputs: (data) =>
		data.class === null
			? []
			: [{ id: "newid", type: `Reference(${data.class})` }],

	serialize: (node) => {
		if (node.data.class === null || node.data.dataset === null) {
			return null;
		}

		return {
			dataset: {
				parameter_type: "Integer",
				value: node.data.dataset,
			},
			class: {
				parameter_type: "Integer",
				value: node.data.class,
			},
		};
	},

	deserialize: async (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const res = await edgeclient.GET("/dataset/list");
		if (res.response.status === 401) {
			location.replace("/");
		} else if (res.response.status !== 200) {
			return null;
		}

		const dataset = serialized.params.dataset;
		if (dataset?.parameter_type !== "Integer") {
			return null;
		}
		const datasetinfo = res.data?.find((x) => x.id === dataset.value) || null;

		const cls = serialized.params.class;
		if (cls?.parameter_type !== "Integer") {
			return null;
		}
		const clsinfo =
			datasetinfo?.classes.find((x) => x.id === cls.value) || null;

		return {
			dataset: datasetinfo?.id || null,
			class: clsinfo?.id || null,

			inputs:
				clsinfo?.attributes.map((x) => ({
					id: x.name,
					tooltip: x.name,

					type:
						x.data_type.type === "Reference"
							? `Reference(${x.data_type.class})`
							: x.data_type.type,
				})) || [],
		};
	},
};
