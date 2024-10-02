import { attrTypes } from "@/lib/attributes";
import { NodeDef } from ".";
import { BaseNode } from "./base";
import { Node, NodeProps, useReactFlow } from "@xyflow/react";
import { Select } from "@mantine/core";

type IfNoneNodeType = Node<
	{
		type: (typeof attrTypes)[number]["serialize_as"];
	},
	"ifnone"
>;

function IfNoneNodeElement({ data, id }: NodeProps<IfNoneNodeType>) {
	const { updateNodeData } = useReactFlow();
	const types = attrTypes.map((x) => x.serialize_as);

	return (
		<>
			<BaseNode
				id={id}
				title={"IfNone"}
				inputs={[
					{ id: "in", type: data.type, tooltip: "Input data" },
					{ id: "ifnone", type: data.type, tooltip: "Fallback if none" },
				]}
				outputs={[{ id: "out", type: data.type, tooltip: "Checksum" }]}
			>
				<Select
					clearable={false}
					label="Data type"
					placeholder="Pick value"
					data={types}
					onChange={(value) => {
						if (value !== null) {
							updateNodeData(id, { type: value });
						}
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

	initialData: {
		type: "Text",
	},

	serialize: (node) => ({
		type: { parameter_type: "String", value: node.data.type },
	}),

	deserialize: (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const t = serialized.params.type;
		if (t?.parameter_type !== "String") {
			return null;
		}

		return {
			type: t.value,
		};
	},
};
