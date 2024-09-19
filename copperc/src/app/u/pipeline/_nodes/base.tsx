import { ReactNode } from "react";
import { Node, useReactFlow } from "@xyflow/react";
import style from "./nodes.module.scss";
import { ActionIcon } from "@mantine/core";
import { Trash2 } from "lucide-react";

function EmptyMarker() {
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

export function BaseNode(params: {
	title: string;
	id: Node["id"];
	children?: ReactNode;
}) {
	const { deleteElements } = useReactFlow();

	return (
		<>
			<div className={style.node_body}>
				<div className={style.node_top}>
					<div className={style.node_top_label}>{params.title}</div>
					<div className={style.node_top_delete}>
						<ActionIcon
							variant="subtle"
							color="white"
							onClick={() => {
								deleteElements({ nodes: [{ id: params.id }] });
							}}
						>
							<Trash2 strokeWidth={2} size="1.5rem" />
						</ActionIcon>
					</div>
				</div>
				<div className={style.node_content}>
					{params.children === undefined ? <EmptyMarker /> : params.children}
				</div>
			</div>
		</>
	);
}
