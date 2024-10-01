import { Select } from "@mantine/core";
import { Node, NodeProps } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";
import { dataTypes } from "@/lib/attributes";

// The "input" node class is already taken

type InputNodetype = Node<Record<string, never>, "pipelineinput">;

export function InputNode({ id }: NodeProps<InputNodetype>) {
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
