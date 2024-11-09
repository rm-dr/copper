import { NodeDef, PipelineDataType } from ".";
import { BaseNode } from "./base";
import { Node, NodeProps, useReactFlow } from "@xyflow/react";
import { Select } from "@mantine/core";
import { attrDataTypes } from "@/lib/attributes";

type IfNoneNodeType = Node<
	{
		type: PipelineDataType;
	},
	"ifnone"
>;

function IfNoneNodeElement({ data, id }: NodeProps<IfNoneNodeType>) {
	const { getEdges, deleteElements, updateNodeData } = useReactFlow();

	return (
		<>
			<BaseNode
				id={id}
				title={"IfNone"}
				inputs={[
					{ id: "in", type: data.type, tooltip: "Input data" },
					{ id: "ifnone", type: data.type, tooltip: "Fallback if none" },
				]}
				outputs={[{ id: "out", type: data.type, tooltip: "Data or fallback" }]}
			>
				<Select
					clearable={false}
					label="Data type"
					placeholder="Pick value"
					data={attrDataTypes}
					onChange={(value) => {
						if (value === null || value === data.type) {
							return;
						}

						updateNodeData(id, { type: value });
						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});
					}}
					value={data.type}
				/>
			</BaseNode>
		</>
	);
}

export const IfNoneNode: NodeDef<IfNoneNodeType> = {
	xyflow_node_type: "ifnone",
	copper_node_type: "IfNone",
	node: IfNoneNodeElement,

	getInputs: (data) => [
		{ id: "in", type: data.type },
		{ id: "ifnone", type: data.type },
	],
	getOutputs: (data) => [{ id: "out", type: data.type }],

	initialData: {
		type: "Text",
	},

	serialize: (node) => ({
		type: { parameter_type: "String", value: node.data.type },
	}),

	deserialize: async (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const t = serialized.params.type;
		if (t?.parameter_type === undefined) {
			return null;
		}

		return {
			type: t.value as PipelineDataType,
		};
	},
};
