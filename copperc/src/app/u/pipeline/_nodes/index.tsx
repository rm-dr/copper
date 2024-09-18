import { ConstantNode } from "./constant";
import { HashNode } from "./hash";
import { IfNoneNode } from "./ifnone";

export const nodeTypes = {
	constant: ConstantNode,
	ifnone: IfNoneNode,
	hash: HashNode,
} as const;

export function EmptyMarker() {
	return (
		<div
			style={{
				width: "100%",
				textAlign: "center",
				fontWeight: 800,
				color: "var(--mantine-color-dimmed)",
			}}
		>
			This node has no options.
		</div>
	);
}
