import { Select } from "@mantine/core";
import {
	Node,
	NodeProps,
	useReactFlow,
	useUpdateNodeInternals,
} from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";
import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";

type AddItemNodeType = Node<
	{
		color: "var(--mantine-primary-color-5)";
	},
	"additem"
>;

export function AddItemNode({ id }: NodeProps<AddItemNodeType>) {
	const [dataset, setDataset] = useState<null | number>(null);
	const [cls, setCls] = useState<null | number>(null);
	const updateNodeInternals = useUpdateNodeInternals();
	const { deleteElements, getEdges } = useReactFlow();

	const list = useQuery({
		queryKey: ["dataset/list"],

		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/list");
			if (res.response.status !== 200) {
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
					list.data === undefined || dataset === null || cls === null
						? undefined
						: list.data
								.find((x) => x.id === dataset)
								?.classes.find((x) => x.id === cls)
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
					error={dataset !== null ? undefined : "No dataset selected"}
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
							setDataset(null);
						} else {
							try {
								setDataset(parseInt(value));
							} catch {}
						}

						updateNodeInternals(id);
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={dataset?.toString() || null}
				/>

				<Select
					key={
						/* Make sure we get the right class list for the selected dataset */
						`class-${dataset}`
					}
					label="Select class"
					disabled={list.data === undefined || dataset === null}
					error={cls !== null ? undefined : "No class selected"}
					data={
						list.data === undefined || dataset === null
							? []
							: list.data
									.find((x) => x.id === dataset)
									?.classes.map((x) => ({
										label: x.name,
										value: x.id.toString(),
									}))
					}
					onChange={(value) => {
						if (value === null) {
							setCls(null);
						} else {
							try {
								setCls(parseInt(value));
							} catch {}
						}

						updateNodeInternals(id);
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={cls?.toString()}
				/>
			</BaseNode>
		</>
	);
}
