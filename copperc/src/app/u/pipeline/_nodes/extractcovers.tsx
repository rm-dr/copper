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
	key: "extractcovers",
	node: ExtractCoversNodeElement,

	initialData: {},

	export: () => ({
		node_type: "ExtractCovers",
	}),
};
