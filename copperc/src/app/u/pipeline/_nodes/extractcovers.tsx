import { Node, NodeProps } from "@xyflow/react";
import { BaseNode } from "./base";

type ExtractCoversNodeType = Node<Record<string, never>, "extractcovers">;

export function ExtractCoversNode({ id }: NodeProps<ExtractCoversNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"Extract covers"}
				inputs={[{ id: "data", type: "Binary", tooltip: "Audio data" }]}
				outputs={[{ id: "out", type: "Binary", tooltip: "Cover" }]}
			/>
		</>
	);
}
