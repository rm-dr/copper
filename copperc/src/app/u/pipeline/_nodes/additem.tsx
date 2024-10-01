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
import { NodeDef } from ".";

type AddItemNodeType = Node<
	{
		dataset: null | number;
		class: null | number;
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

			return res.data;
		},
	});

	return (
		<>
			<BaseNode
				id={id}
				title={"Add Item"}
				inputs={
					list.data === undefined ||
					data.dataset === null ||
					data.class === null
						? undefined
						: list.data
								.find((x) => x.id === data.dataset)
								?.classes.find((x) => x.id === data.class)
								?.attributes.map((x) => ({
									id: x.name,
									type: x.data_type.type,
									tooltip: x.name,
								})) || undefined
				}
			>
				<Select
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
							updateNodeData(id, { ...data, dataset: null });
						} else {
							try {
								updateNodeData(id, { ...data, dataset: parseInt(value) });
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
							updateNodeData(id, { ...data, class: null });
						} else {
							try {
								updateNodeData(id, { ...data, class: parseInt(value) });
							} catch {}
						}

						updateNodeInternals(id);
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={data.class?.toString()}
				/>
			</BaseNode>
		</>
	);
}

export const AddItemNode: NodeDef<AddItemNodeType> = {
	key: "additem",
	node: AddItemNodeElement,

	initialData: {
		dataset: null,
		class: null,
	},

	export: (node) => {
		if (node.data.class === null) {
			return null;
		}

		return {
			node_type: "AddItem",
			params: {
				class: {
					parameter_type: "Integer",
					value: node.data.class,
				},
			},
		};
	},
};
