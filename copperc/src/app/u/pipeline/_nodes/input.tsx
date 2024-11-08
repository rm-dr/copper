import { Select } from "@mantine/core";
import { Node, NodeProps, useReactFlow } from "@xyflow/react";
import { BaseNode } from "./base";
import { dataTypes } from "@/lib/attributes";
import { NodeDef } from ".";
import { components } from "@/lib/api/openapi";

const types = ["Text", "Integer", "Float"] as const;

type InputNodeType = Node<
	{
		type: (typeof types)[number];
	},
	"pipelineinput"
>;

function InputNodeElement({ data, id }: NodeProps<InputNodeType>) {
	const { deleteElements, getEdges, updateNodeData } = useReactFlow();

	return (
		<>
			<BaseNode
				id={id}
				title={"Input"}
				outputs={[{ id: "out", tooltip: "Input value", type: data.type }]}
				top_color="var(--mantine-color-green-8)"
			>
				<Select
					label="Data type"
					placeholder="Pick value"
					data={dataTypes}
					onChange={(value) => {
						if (value === null || value === data.type) {
							return;
						}

						if (
							dataTypes.includes(
								value as components["schemas"]["AttrDataStub"]["type"],
							)
						) {
							updateNodeData(id, {
								type: value,
							});
						}

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

export const InputNode: NodeDef<InputNodeType> = {
	// The "input" node class is already taken
	xyflow_node_type: "pipelineinput",
	copper_node_type: "Input",
	node: InputNodeElement,

	getInputs: () => [],
	getOutputs: (data) => [{ id: "out", type: data.type }],

	minimap_color: "var(--mantine-color-green-8)",

	initialData: { type: "Text" },

	serialize: (node) => ({
		input_name: {
			parameter_type: "String",
			value: node.id,
		},

		input_type: {
			parameter_type: "String",
			value: node.data.type,
		},
	}),

	deserialize: async (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const data_type = serialized.params.input_type;
		if (data_type?.parameter_type !== "String") {
			return null;
		}

		return {
			type: data_type.value as (typeof types)[number],
		};
	},
};
