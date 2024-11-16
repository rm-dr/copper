import { NumberInput, Select, TextInput } from "@mantine/core";
import { Node, NodeProps, useReactFlow } from "@xyflow/react";
import { BaseNode } from "./base";
import { NodeDef } from ".";
import { stringUnionToArray } from "@/lib/util";

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
	  }
	| {
			type: "Boolean";
			value: boolean;
	  };

type ConstType = ValueType["type"];

export const constTypes = stringUnionToArray<ConstType>()(
	"Text",
	"Integer",
	"Float",
	"Boolean",
);

type ConstantNodeType = Node<
	{
		value: ValueType;
	},
	"constant"
>;

function ConstantNodeElement({ data, id }: NodeProps<ConstantNodeType>) {
	const { deleteElements, getEdges, updateNodeData } = useReactFlow();

	let input = null;
	if (data.value.type === "Text") {
		input = (
			<TextInput
				label="Value"
				placeholder="enter constant text"
				onChange={(event) =>
					updateNodeData(id, {
						value: {
							type: "Text",
							value: event.currentTarget.value,
						},
					})
				}
				value={data.value.value}
			/>
		);
	} else if (data.value.type === "Integer") {
		input = (
			<NumberInput
				label="Value"
				placeholder="enter constant integer"
				onChange={(value) => {
					if (typeof value !== "number") {
						return;
					}

					updateNodeData(id, {
						value: {
							type: "Integer",
							value,
						},
					});
				}}
				value={data.value.value}
				allowDecimal={false}
			/>
		);
	} else if (data.value.type === "Float") {
		input = (
			<NumberInput
				label="Value"
				placeholder="enter constant float"
				onChange={(value) => {
					if (typeof value !== "number") {
						return;
					}

					updateNodeData(id, {
						value: {
							type: "Float",
							value,
						},
					});
				}}
				value={data.value.value}
				allowDecimal={true}
			/>
		);
	} else if (data.value.type === "Boolean") {
		input = (
			<Select
				label="Value"
				placeholder="enter constant boolean"
				data={["true", "false"]}
				value={data.value.value ? "true" : "false"}
				onChange={(value) => {
					updateNodeData(id, {
						value: {
							type: "Boolean",
							value: value === "true",
						},
					});
				}}
			/>
		);
	}

	return (
		<>
			<BaseNode
				id={id}
				title={"Constant"}
				outputs={[
					{ id: "out", tooltip: "Output value", type: data.value.type },
				]}
			>
				<Select
					label="Data type"
					placeholder="Pick value"
					data={constTypes}
					onChange={(v) => {
						const value = v as ConstType | null;

						if (value === null || value == data.value.type) {
							return;
						}

						deleteElements({
							edges: getEdges()
								.filter((x) => x.source === id || x.target === id)
								.map((x) => ({ id: x.id })),
						});

						if (value === "Text") {
							updateNodeData(id, {
								value: {
									type: "Text",
									value: "",
								},
							});
						} else if (value === "Integer") {
							updateNodeData(id, {
								value: {
									type: "Integer",
									value: 0,
								},
							});
						} else if (value === "Float") {
							updateNodeData(id, {
								value: {
									type: "Float",
									value: 0,
								},
							});
						} else if (value === "Boolean") {
							updateNodeData(id, {
								value: {
									type: "Boolean",
									value: false,
								},
							});
						}
					}}
					value={data.value.type}
				/>

				{input}
			</BaseNode>
		</>
	);
}

export const ConstantNode: NodeDef<ConstantNodeType> = {
	xyflow_node_type: "constant",
	copper_node_type: "Constant",
	node: ConstantNodeElement,

	initialData: {
		value: {
			type: "Text",
			value: "",
		},
	},

	getInputs: () => [],
	getOutputs: (data) => [{ id: "out", type: data.value.type }],

	serialize: (node) => {
		if (node.data.value.type === "Text") {
			return {
				value: { parameter_type: "String", value: node.data.value.value },
			};
		} else if (node.data.value.type === "Integer") {
			return {
				value: { parameter_type: "Integer", value: node.data.value.value },
			};
		}

		throw new Error(
			`Entered unreachable code: unhandled type ${node.data.value} in constant node`,
		);
	},

	deserialize: async (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const v = serialized.params.value;
		if (v?.parameter_type === "String") {
			return {
				value: {
					type: "Text",
					value: v.value,
				},
			};
		} else if (v?.parameter_type === "Integer") {
			return {
				value: {
					type: "Integer",
					value: v.value,
				},
			};
		}

		return null;
	},
};
