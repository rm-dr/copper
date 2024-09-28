import { BaseNode } from "./base";
import { Node, NodeProps } from "@xyflow/react";

type IfNoneNodeType = Node<Record<string, never>, "ifnone">;

export function IfNoneNode({ id }: NodeProps<IfNoneNodeType>) {
	return (
		<>
			<BaseNode
				id={id}
				title={"IfNone"}
				inputs={[
					{ id: "in", type: "Unknown", tooltip: "Input data" },
					{ id: "ifnone", type: "Unknown", tooltip: "Fallback if none" },
				]}
				outputs={[{ id: "out", type: "Unknown", tooltip: "Checksum" }]}
			/>
		</>
	);
}
