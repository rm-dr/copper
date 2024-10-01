import { Select } from "@mantine/core";
import { Node, NodeProps, useReactFlow } from "@xyflow/react";
import { BaseNode } from "./base";
import { NodeDef } from ".";

const HashTypes = ["SHA256", "SHA512", "MD5"] as const;

type HashNodeType = Node<
	{
		hash_type: (typeof HashTypes)[number];
	},
	"hash"
>;

function HashNodeElement({ data, id }: NodeProps<HashNodeType>) {
	const { updateNodeData } = useReactFlow();

	return (
		<>
			<BaseNode
				id={id}
				title={"Checksum"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Input data" }]}
				outputs={[{ id: "hash", type: "Hash", tooltip: "Checksum" }]}
			>
				<Select
					label="Hash type"
					data={HashTypes}
					onChange={(value) => {
						// eslint-disable-next-line
						if (value !== null && HashTypes.includes(value as any)) {
							updateNodeData(id, {
								hashType: value as (typeof HashTypes)[number],
							});
						}
					}}
					value={data.hash_type}
				/>
			</BaseNode>
		</>
	);
}

export const HashNode: NodeDef<HashNodeType> = {
	key: "hash",
	node_type: "Hash",
	node: HashNodeElement,

	initialData: {
		hash_type: "SHA256",
	},

	serialize: (node) => ({
		hash_type: { parameter_type: "String", value: node.data.hash_type },
	}),

	deserialize: (serialized) => {
		if (serialized.params === undefined) {
			return null;
		}

		const hash_type = serialized.params.hash_type;
		if (hash_type?.parameter_type !== "String") {
			return null;
		}

		if (!HashTypes.includes(hash_type.value as (typeof HashTypes)[number])) {
			return null;
		}

		return {
			hash_type: hash_type.value as (typeof HashTypes)[number],
		};
	},
};
