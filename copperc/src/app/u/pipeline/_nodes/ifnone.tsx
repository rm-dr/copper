import { EmptyMarker } from ".";
import style from "./nodes.module.scss";
import { Handle, Position } from "@xyflow/react";

export function IfNoneNode({}) {
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

			<div className={style.node_body}>
				<div className={style.node_label}>
					<label>IfNone</label>
				</div>
				<div className={style.node_content}>
					<EmptyMarker />
				</div>
			</div>

			<Handle
				className={style.node_handle}
				type="source"
				position={Position.Right}
				id="out"
			/>
		</>
	);
}
