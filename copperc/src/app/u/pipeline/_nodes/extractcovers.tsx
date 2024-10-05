import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";
import { NodeDef } from ".";

type ExtractCoversNodeType = Node<Record<string, never>, "extractcovers">;

function ExtractCoversNodeElement({ id }: NodeProps<ExtractCoversNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Extract covers"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Audio data" }]}
				outputs={[{ id: "out", type: "Blob", tooltip: "Cover" }]}
			/>
		</>
	);
}

export const ExtractCoversNode: NodeDef<ExtractCoversNodeType> = {
	xyflow_node_type: "extractcovers",
	copper_node_type: "ExtractCovers",
	node: ExtractCoversNodeElement,

	getInputs: () => [{ id: "data", type: "Blob" }],
	getOutputs: () => [{ id: "out", type: "Blob" }],

	initialData: {},
	serialize: () => ({}),
	deserialize: async () => ({}),
};
