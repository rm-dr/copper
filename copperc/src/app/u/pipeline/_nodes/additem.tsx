import { Select } from "@mantine/core";
import { Node, NodeProps } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";

type AddItemNodeType = Node<
	{
		color: "var(--mantine-primary-color-5)";
	},
	"additem"
>;

export function AddItemNode({ id }: NodeProps<AddItemNodeType>) {
	const [value, setValue] = useState<null | number>(null);

	return (
		<>
			<BaseNode id={id} title={"Add Item"}>
				<Select
					label="Data type"
					placeholder="Pick value"
					data={["1", "5"]}
					onChange={(value) => {
						if (value === null) {
							return;
						}

						try {
							setValue(parseInt(value));
						} catch {}
					}}
					value={value?.toString()}
				/>
			</BaseNode>
		</>
	);
}
