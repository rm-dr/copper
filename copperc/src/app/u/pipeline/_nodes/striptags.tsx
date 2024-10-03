import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";
import { NodeDef } from ".";

type StripTagsNodeType = Node<Record<string, never>, "striptags">;

function StripTagsNodeElement({ id }: NodeProps<StripTagsNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Strip tags"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Audio data" }]}
				outputs={[
					{
						id: "out",
						type: "Blob",
						tooltip: "Audio data with tags stripped",
					},
				]}
			/>
		</>
	);
}

export const StripTagsNode: NodeDef<StripTagsNodeType> = {
	xyflow_node_type: "striptags",
	copper_node_type: "StripTags",
	node: StripTagsNodeElement,

	getInputs: () => [{ id: "data", type: "Blob" }],
	getOutputs: () => [{ id: "out", type: "Blob" }],

	initialData: {},
	serialize: () => ({}),
	deserialize: () => ({}),
};
