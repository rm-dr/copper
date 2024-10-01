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
	key: "striptags",
	node: StripTagsNodeElement,
};
