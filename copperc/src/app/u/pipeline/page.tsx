"use client";
import React, { Dispatch, SetStateAction, useCallback } from "react";
import { Node, ReactFlowProvider } from "@xyflow/react";

import style from "./pipeline.module.scss";
import nodestyle from "./_nodes/nodes.module.scss";
import "@xyflow/react/dist/style.css";

import { useFlow } from "./flow";
import { ActionIcon, Button, Select } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { InfoIcon } from "lucide-react";
import { nodeDefinitions } from "./_nodes";

function AddNodeButton(params: {
	text: string;
	node_type: string;

	setNodes: Dispatch<SetStateAction<Node[]>>;
	onInfo: () => void;
}) {
	const node = nodeDefinitions[params.node_type];
	if (node === undefined) {
		console.error(`Unknown node type ${params.node_type}`);
		return;
	}

	return (
		<div className={style.add_node_button}>
			<ActionIcon
				variant="transparent"
				aria-label="Settings"
				onClick={params.onInfo}
			>
				<InfoIcon size={"1rem"} />
			</ActionIcon>
			<Button
				fullWidth
				variant="light"
				size="xs"
				onClick={() => {
					const id = getId();

					const newNode: Node = {
						id,
						type: params.node_type,
						position: { x: 0, y: 0 },
						data: node.initialData,
						origin: [0.5, 0.0],
						dragHandle: `.${nodestyle.node_top_label}`,
					};

					params.setNodes((nodes) => nodes.concat(newNode));
				}}
			>
				{params.text}
			</Button>
		</div>
	);
}

let id = 1;
const getId = () => `node-${id++}`;

function Main() {
	const { flow, getFlow, setNodes } = useFlow();

	const savePipeline = useCallback(() => {
		const raw = getFlow();
		console.log(raw);

		if (raw === null) {
			return;
		}

		const nodes: components["schemas"]["PipelineJson"]["nodes"] = {};
		raw.nodes.forEach((node) => {
			if (node.type === undefined) {
				return;
			}

			const nodedef = nodeDefinitions[node.type];
			if (nodedef === undefined) {
				return;
			}

			const ex = nodedef.export(node);
			if (ex === null) {
				return;
			}

			nodes[node.id] = ex;
		});

		const edges: components["schemas"]["PipelineJson"]["edges"] = {};
		// eslint-disable-next-line
		raw.edges.forEach((edge: any) => {
			const source: components["schemas"]["InputPort"] = {
				node: edge.source,
				port: "",
			};

			const target: components["schemas"]["OutputPort"] = {
				node: edge.target,
				port: "",
			};

			edge[edge.id] = {
				source,
				target,
			};
		});

		const pipe: components["schemas"]["PipelineJson"] = {
			nodes,
			edges,
		};

		console.log(pipe);
	}, [getFlow]);

	return (
		<div className={style.pipeline_container}>
			<div className={style.tools_container}>
				<div className={style.tools_section}>
					<div className={style.tools_section_title}>Select pipeline</div>

					<Select data={["a"]} onChange={() => {}} />

					<Button.Group style={{ width: "100%" }}>
						<Button fullWidth variant="subtle" size="xs">
							Reload
						</Button>

						<Button fullWidth variant="subtle" size="xs">
							Rename
						</Button>
					</Button.Group>

					{false ? (
						<Button
							fullWidth
							variant="subtle"
							size="xs"
							color="red"
							onClick={savePipeline}
						>
							Save (!)
						</Button>
					) : (
						<Button fullWidth variant="subtle" size="xs" onClick={savePipeline}>
							Save
						</Button>
					)}
				</div>

				<div className={style.tools_section}>
					<div className={style.tools_section_title}>Add nodes</div>

					<div className={style.node_group}>
						<div className={style.node_group_title}>Base</div>
						<AddNodeButton
							text="Input"
							node_type="pipelineinput"
							setNodes={setNodes}
							onInfo={() => {}}
						/>

						<AddNodeButton
							text="Constant"
							node_type="constant"
							setNodes={setNodes}
							onInfo={() => {}}
						/>

						<AddNodeButton
							text="IfNone"
							node_type="ifnone"
							setNodes={setNodes}
							onInfo={() => {}}
						/>

						<AddNodeButton
							text="Checksum"
							node_type="hash"
							setNodes={setNodes}
							onInfo={() => {}}
						/>
					</div>

					<div className={style.node_group}>
						<div className={style.node_group_title}>Storage</div>
						<AddNodeButton
							text="Add item"
							node_type="additem"
							setNodes={setNodes}
							onInfo={() => {}}
						/>
					</div>

					<div className={style.node_group}>
						<div className={style.node_group_title}>Audio</div>
						<AddNodeButton
							text="Strip tags"
							node_type="striptags"
							setNodes={setNodes}
							onInfo={() => {}}
						/>
						<AddNodeButton
							text="Extract tags"
							node_type="extracttags"
							setNodes={setNodes}
							onInfo={() => {}}
						/>
						<AddNodeButton
							text="Extract covers"
							node_type="extractcovers"
							setNodes={setNodes}
							onInfo={() => {}}
						/>
					</div>
				</div>
			</div>
			<div className={style.react_flow_container}>{flow}</div>
		</div>
	);
}

export default function Page() {
	return (
		<ReactFlowProvider>
			<Main />
		</ReactFlowProvider>
	);
}
