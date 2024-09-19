import { BaseNode } from "./base";
import style from "./nodes.module.scss";
import { Handle, Node, NodeProps, Position } from "@xyflow/react";

type IfNoneNodeType = Node<Record<string, never>, "ifnone">;

export function IfNoneNode({ id }: NodeProps<IfNoneNodeType>) {
	return (
		<>
			<Handle
				className={style.node_handle}
				type="target"
				position={Position.Left}
				id="in"
				style={{ top: 10 }}
			/>

			<Handle
				className={style.node_handle}
				type="target"
				position={Position.Left}
				id="ifnone"
			/>

			<BaseNode id={id} title={"IfNone"} />

			<Handle
				className={style.node_handle}
				type="source"
				position={Position.Right}
				id="out"
			/>
		</>
	);
}
