import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";

type StripTagsNodeType = Node<Record<string, never>, "striptags">;

export function StripTagsNode({ id }: NodeProps<StripTagsNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Strip tags"}
				inputs={[{ id: "data", type: "Binary", tooltip: "Audio data" }]}
				outputs={[
					{
						id: "out",
						type: "Binary",
						tooltip: "Audio data with tags stripped",
					},
				]}
			/>
		</>
	);
}
