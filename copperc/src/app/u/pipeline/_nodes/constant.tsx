import { NumberInput, Select, TextInput } from "@mantine/core";
import { Node, NodeProps } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";
import { NodeDef } from ".";

type ConstantNodeType = Node<
	{
		color: "var(--mantine-primary-color-5)";
	},
	"constant"
>;

function ConstantNodeElement({ id }: NodeProps<ConstantNodeType>) {
	const types = ["Text", "Integer", "Float"] as const;
	type ValueType =
		| {
				type: "Text";
				value: string;
		  }
		| {
				type: "Integer";
				value: number;
		  }
		| {
				type: "Float";
				value: number;
		  };

	const [value, setValue] = useState<ValueType>({ type: "Text", value: "" });

	let input = null;
	if (value.type === "Text") {
		input = (
			<TextInput
				label="Value"
				placeholder="enter constant text"
				onChange={(event) =>
					setValue({
						type: "Text",
						value: event.currentTarget.value,
					})
				}
				value={value.value}
			/>
		);
	} else if (value.type === "Integer") {
		input = (
			<NumberInput
				label="Value"
				placeholder="enter constant number"
				onChange={(value) => {
					if (typeof value === "string") {
						return;
					}

					setValue({
						type: "Integer",
						value,
					});
				}}
				value={value.value}
				allowDecimal={false}
			/>
		);
	} else if (value.type === "Float") {
		input = (
			<NumberInput
				label="Value"
				placeholder="enter constant number"
				onChange={(value) => {
					if (typeof value === "string") {
						return;
					}

					setValue({
						type: "Integer",
						value,
					});
				}}
				value={value.value}
				allowDecimal={true}
			/>
		);
	}

	return (
		<>
			<BaseNode
				id={id}
				title={"Constant"}
				outputs={[{ id: "out", tooltip: "Output value", type: value.type }]}
			>
				<Select
					label="Data type"
					placeholder="Pick value"
					data={types}
					onChange={(value) => {
						if (value === "Text") {
							setValue({
								type: "Text",
								value: "",
							});
						} else if (value === "Integer") {
							setValue({
								type: "Integer",
								value: 0,
							});
						}
					}}
					value={value.type}
				/>

				{input}
			</BaseNode>
		</>
	);
}

export const ConstantNode: NodeDef<ConstantNodeType> = {
	key: "constant",
	node: ConstantNodeElement,
};
