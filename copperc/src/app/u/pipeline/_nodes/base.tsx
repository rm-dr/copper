import { ReactNode } from "react";
import { Handle, Node, Position, useReactFlow } from "@xyflow/react";
import style from "./nodes.module.scss";
import { ActionIcon } from "@mantine/core";
import { Trash2 } from "lucide-react";
import { PipelineDataType } from ".";
import { AttrDataType, getAttrTypeInfo } from "@/lib/attributes";

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
	top_color?: string;

	outputs?: {
		type: PipelineDataType;
		id: string;
		tooltip: string;
	}[];

	inputs?: {
		type: PipelineDataType;
		id: string;
		tooltip: string;
	}[];
}) {
	const { deleteElements } = useReactFlow();

	return (
		<>
			<div className={style.node_body}>
				<div
					className={style.node_top}
					style={
						params.top_color === undefined
							? { background: "var(--mantine-primary-color-5)" }
							: { background: params.top_color }
					}
				>
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
					{params.inputs === undefined || params.inputs.length === 0 ? null : (
						<div
							className={`${style.node_inputs} ${style.node_port_container} ${style.input}`}
						>
							{params.inputs.map((x) => {
								return (
									<div
										key={`handle-${x.id}`}
										className={`${style.node_port} ${style.input}`}
									>
										{
											// Convert `Reference(number)` into `number
											x.type.startsWith("Reference")
												? getAttrTypeInfo("Reference").icon
												: getAttrTypeInfo(x.type as AttrDataType).icon
										}
										<Handle type="target" position={Position.Left} id={x.id} />
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

					{params.outputs === undefined ||
					params.outputs.length === 0 ? null : (
						<div
							className={`${style.node_outputs} ${style.node_port_container} ${style.output}`}
						>
							{params.outputs.map((x) => {
								return (
									<div
										key={`handle-${x.id}`}
										className={`${style.node_port} ${style.output}`}
									>
										{
											// Convert `Reference(number)` into `number
											x.type.startsWith("Reference")
												? getAttrTypeInfo("Reference").icon
												: getAttrTypeInfo(x.type as AttrDataType).icon
										}
										<Handle type="source" position={Position.Right} id={x.id} />
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
