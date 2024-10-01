import { Select } from "@mantine/core";
import { Node, NodeProps } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";
import { dataTypes } from "@/lib/attributes";
import { NodeDef } from ".";

type InputNodeType = Node<Record<string, never>, "pipelineinput">;

function InputNodeElement({ id }: NodeProps<InputNodeType>) {
	const types = ["Text", "Integer", "Float"] as const;

	const [valuetype, setValueType] = useState<(typeof types)[number]>("Text");

	return (
		<>
			<BaseNode
				id={id}
				title={"Input"}
				outputs={[{ id: "out", tooltip: "Input value", type: valuetype }]}
			>
				<Select
					label="Data type"
					placeholder="Pick value"
					data={dataTypes}
					onChange={(value) => {
						if (value === null) {
							return;
						}

						if (dataTypes.includes(value)) {
							setValueType(value as (typeof types)[number]);
						}
					}}
					value={valuetype}
				/>
			</BaseNode>
		</>
	);
}

export const InputNode: NodeDef<InputNodeType> = {
	// The "input" node class is already taken
	key: "pipelineinput",
	node_type: "Input",
	node: InputNodeElement,

	initialData: {},

	serialize: (node) => ({
		input_name: {
			parameter_type: "String",
			value: node.id,
		},
	}),

	deserialize: () => ({}),
};
