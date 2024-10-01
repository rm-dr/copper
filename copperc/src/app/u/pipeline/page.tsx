"use client";
import React, { Dispatch, SetStateAction, useCallback, useState } from "react";
import {
	Edge,
	Node,
	ReactFlowJsonObject,
	ReactFlowProvider,
	useReactFlow,
} from "@xyflow/react";

import style from "./pipeline.module.scss";
import nodestyle from "./_nodes/nodes.module.scss";
import "@xyflow/react/dist/style.css";

import { useFlow } from "./flow";
import { ActionIcon, Button, Select } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { InfoIcon } from "lucide-react";
import { nodeDefinitions } from "./_nodes";
import { useAddPipelineModal } from "./_modals/addpipeline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { useDeletePipelineModal } from "./_modals/deletepipeline";
import { useRenamePipelineModal } from "./_modals/renamepipeline";

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

/**
 * Generate a unique node id
 */
function getId(): string {
	const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
	const length = 10;

	let rand = "";
	const charactersLength = characters.length;
	for (let i = 0; i < length; i++) {
		rand += characters.charAt(Math.floor(Math.random() * charactersLength));
	}

	return `node-${rand}-${new Date().valueOf()}`;
}

function PipelineButtons(params: {
	pipeline: components["schemas"]["PipelineInfo"];
	getFlow: () => ReactFlowJsonObject<Node, Edge> | null;
	onChange: (select: components["schemas"]["PipelineInfo"] | null) => void;
}) {
	const { setViewport, setNodes, setEdges } = useReactFlow();

	const { open: openDeletePipeline, modal: modalDeletePipeline } =
		useDeletePipelineModal({
			pipeline_id: params.pipeline.id,
			pipeline_name: params.pipeline.name,
			onSuccess: () => params.onChange(null),
		});

	const { open: openRenamePipeline, modal: modalRenamePipeline } =
		useRenamePipelineModal({
			pipeline_id: params.pipeline.id,
			pipeline_name: params.pipeline.name,
			onSuccess: params.onChange,
		});

	const doSave = useMutation({
		mutationFn: async (new_data: components["schemas"]["PipelineJson"]) => {
			return await edgeclient.PATCH("/pipeline/{pipeline_id}", {
				params: { path: { pipeline_id: params.pipeline.id } },
				body: { new_data },
			});
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				params.onChange(res.data!);
			} else {
				throw new Error(res.error);
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const savePipeline = useCallback(() => {
		const raw = params.getFlow();
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

			const ex = nodedef.serialize(node);
			if (ex === null) {
				return;
			}

			nodes[node.id] = {
				node_type: nodedef.node_type,
				position: node.position,
				params: ex,
			};
		});

		const edges: components["schemas"]["PipelineJson"]["edges"] = {};
		raw.edges.forEach((edge) => {
			let sourcePort = edge.sourceHandle;
			if (sourcePort === null || sourcePort === undefined) {
				const node = raw.nodes.find((x) => x.id === edge.source);
				if (node === undefined) {
					console.error(
						`Could not find node ${edge.source} (referenced by edge ${edge.id})`,
					);
					return;
				}

				if (node.handles === undefined || node.handles.length === 0) {
					console.error(
						`Node ${edge.source} has no handles, but is connected to an edge ${edge.id}`,
					);
					return;
				}

				const firsthandle = node.handles[0];
				if (firsthandle === undefined || node.handles.length !== 1) {
					console.error(
						`Edge ${edge.id} does not give an explicit handle for node ${edge.source}, but that node has multiple edges.`,
					);
					return;
				}

				sourcePort = firsthandle.id;

				if (sourcePort === null || sourcePort === undefined) {
					console.error(
						`Handle of ${edge.id} on ${edge.source} doesn't have an id.`,
					);
					return;
				}
			}
			const source: components["schemas"]["InputPort"] = {
				node: edge.source,
				port: sourcePort,
			};

			let targetPort = edge.targetHandle;
			if (targetPort === null || targetPort === undefined) {
				const node = raw.nodes.find((x) => x.id === edge.target);
				if (node === undefined) {
					console.error(
						`Could not find node ${edge.target} (referenced by edge ${edge.id})`,
					);
					return;
				}

				if (node.handles === undefined || node.handles.length === 0) {
					console.error(
						`Node ${edge.target} has no handles, but is connected to an edge ${edge.id}`,
					);
					return;
				}

				const firsthandle = node.handles[0];
				if (firsthandle === undefined || node.handles.length !== 1) {
					console.error(
						`Edge ${edge.id} does not give an explicit handle for node ${edge.target}, but that node has multiple edges.`,
					);
					return;
				}

				targetPort = firsthandle.id;

				if (targetPort === null || targetPort === undefined) {
					console.error(
						`Handle of ${edge.id} on ${edge.target} doesn't have an id.`,
					);
					return;
				}
			}
			const target: components["schemas"]["OutputPort"] = {
				node: edge.target,
				port: targetPort,
			};

			edges[edge.id] = {
				source,
				target,
			};
		});

		console.log(edges);

		doSave.mutate({
			nodes,
			edges,
		});
	}, [doSave, params]);

	return (
		<>
			{modalDeletePipeline}
			{modalRenamePipeline}
			<Button.Group style={{ width: "100%" }}>
				<Button
					fullWidth
					variant="subtle"
					size="xs"
					onClick={openDeletePipeline}
					disabled={doSave.isPending}
				>
					Delete
				</Button>

				<Button
					fullWidth
					variant="subtle"
					size="xs"
					onClick={openRenamePipeline}
					disabled={doSave.isPending}
				>
					Rename
				</Button>
			</Button.Group>

			<Button.Group style={{ width: "100%" }}>
				<Button
					fullWidth
					variant="subtle"
					size="xs"
					disabled={doSave.isPending}
					onClick={() => {
						setNodes(
							Object.entries(params.pipeline.data.nodes)
								.map((x) => {
									const v = x[1];

									const nodedef = Object.entries(nodeDefinitions).find(
										(x) => x[1].node_type === v.node_type,
									);
									if (nodedef === undefined) {
										console.error(`Unknown node type ${v.node_type}`);
										return null;
									}

									const des = nodedef[1].deserialize(v);
									if (des === null) {
										return null;
									}

									const node: Node = {
										id: x[0],
										type: nodedef[1].key,
										position: v.position,
										data: des,
										origin: [0.5, 0.0],
										dragHandle: `.${nodestyle.node_top_label}`,
									};

									return node;
								})
								.filter((x) => x !== null),
						);

						setEdges(
							Object.entries(params.pipeline.data.edges).map((x) => {
								const v = x[1];
								const edge: Edge = {
									type: "default",
									id: x[0],
									source: v.source.node,
									sourceHandle: v.source.port,
									target: v.target.node,
									targetHandle: v.target.port,
								};

								return edge;
							}),
						);

						setViewport({ x: 0, y: 0, zoom: 1 });
					}}
				>
					Reload
				</Button>

				{false ? (
					<Button
						fullWidth
						variant="subtle"
						size="xs"
						color="red"
						onClick={savePipeline}
						loading={doSave.isPending}
					>
						Save (!)
					</Button>
				) : (
					<Button
						fullWidth
						variant="subtle"
						size="xs"
						onClick={savePipeline}
						loading={doSave.isPending}
					>
						Save
					</Button>
				)}
			</Button.Group>
		</>
	);
}

function Main() {
	const { flow, getFlow, setNodes } = useFlow();
	const qc = useQueryClient();
	const [pipeline, setPipeline] = useState<
		components["schemas"]["PipelineInfo"] | null
	>(null);

	const pipelines = useQuery({
		queryKey: ["pipeline/list"],

		queryFn: async () => {
			const res = await edgeclient.GET("/pipeline/list");
			if (res.response.status === 401) {
				location.replace("/");
			}

			if (res.response.status !== 200) {
				throw new Error("could not get pipelines");
			}

			return res.data!;
		},
	});

	const { open: openAddPipeline, modal: modalAddPipeline } =
		useAddPipelineModal({
			onSuccess: (new_info) => {
				setPipeline(new_info);
				qc.invalidateQueries({ queryKey: ["dataset/list"] });
				pipelines.refetch();
			},
		});

	return (
		<>
			{modalAddPipeline}

			<div className={style.pipeline_container}>
				<div className={style.tools_container}>
					<div className={style.tools_section}>
						<div className={style.tools_section_title}>Select pipeline</div>

						<Button
							fullWidth
							variant="subtle"
							size="xs"
							onClick={openAddPipeline}
						>
							New pipeline
						</Button>

						<Select
							disabled={pipelines.data === undefined}
							data={
								pipelines.data === undefined || pipeline?.data === null
									? []
									: pipelines.data.map((x) => ({
											label: x.name,
											value: x.id.toString(),
									  }))
							}
							value={pipeline === null ? null : pipeline.id.toString()}
							onChange={(value) => {
								const int = value === null ? null : parseInt(value);
								if (int === null || pipelines.data === undefined) {
									setPipeline(null);
								}
								setPipeline(pipelines.data?.find((x) => x.id === int) || null);
							}}
						/>

						{pipeline === null ? null : (
							<PipelineButtons
								pipeline={pipeline}
								getFlow={getFlow}
								onChange={(select) => {
									qc.invalidateQueries({ queryKey: ["dataset/list"] });
									pipelines.refetch();
									setPipeline(select);
								}}
							/>
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
		</>
	);
}

export default function Page() {
	return (
		<ReactFlowProvider>
			<Main />
		</ReactFlowProvider>
	);
}
