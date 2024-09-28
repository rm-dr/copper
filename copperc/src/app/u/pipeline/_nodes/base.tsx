import { ReactNode } from "react";
import { Handle, Node, Position, useReactFlow } from "@xyflow/react";
import style from "./nodes.module.scss";
import { ActionIcon } from "@mantine/core";
import {
	Binary,
	Box,
	CircleHelp,
	Hash,
	LetterText,
	Trash2,
} from "lucide-react";

export const DATA_TYPES = [
	"Text",
	"Integer",
	"Float",
	"Binary",
	"Hash",
	"Unknown",
] as const;
export type DataType = (typeof DATA_TYPES)[number];

function type_to_icon(type: DataType) {
	if (type === "Text") {
		return <LetterText />;
	} else if (type === "Float") {
		return <Hash />;
	} else if (type === "Integer") {
		return <Hash />;
	} else if (type === "Binary") {
		return <Binary />;
	} else if (type === "Hash") {
		return <Box />;
	}

	return <CircleHelp />;
}

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

	outputs?: {
		type: DataType;
		id: string;
		tooltip: string;
	}[];

	inputs?: {
		type: DataType;
		id: string;
		tooltip: string;
	}[];
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
							onMouseDown={(e) => {
								if (e.button === 0) {
									deleteElements({ nodes: [{ id: params.id }] });
								}
							}}
						>
							<Trash2 strokeWidth={2} size="1.5rem" />
						</ActionIcon>
					</div>
				</div>
				<div className={style.node_body_inner}>
					{params.inputs === undefined ? null : (
						<div
							className={`${style.node_inputs} ${style.node_port_container} ${style.input}`}
						>
							{params.inputs.map((x) => {
								return (
									<div
										key={`handle-${x}`}
										className={`${style.node_port} ${style.input}`}
									>
										{type_to_icon(x.type)}
										<Handle
											style={{
												width: "1rem",
												height: "1rem",
											}}
											type="target"
											position={Position.Left}
											id={x.id}
										/>

										<div className={`${style.port_tooltip} ${style.input}`}>
											{x.tooltip}
										</div>
									</div>
								);
							})}
						</div>
					)}

					<div className={style.node_content}>
						{params.children === undefined ? <EmptyMarker /> : params.children}
					</div>

					{params.outputs === undefined ? null : (
						<div
							className={`${style.node_outputs} ${style.node_port_container} ${style.output}`}
						>
							{params.outputs.map((x) => {
								return (
									<div
										key={`handle-${x}`}
										className={`${style.node_port} ${style.output}`}
									>
										{type_to_icon(x.type)}
										<Handle
											style={{
												width: "1rem",
												height: "1rem",
											}}
											type="source"
											position={Position.Right}
											id={x.id}
										/>

										<div className={`${style.port_tooltip} ${style.output}`}>
											{x.tooltip}
										</div>
									</div>
								);
							})}
						</div>
					)}
				</div>
			</div>
		</>
	);
}
