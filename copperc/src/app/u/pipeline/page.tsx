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
import { ActionIcon, Button, Select, Text } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { InfoIcon } from "lucide-react";
import { nodeDefinitions } from "./_nodes";
import { useAddPipelineModal } from "./_modals/addpipeline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { useDeletePipelineModal } from "./_modals/deletepipeline";
import { useRenamePipelineModal } from "./_modals/renamepipeline";
import { deserializePipeline, serializePipeline } from "./serde";

function AddNodeButton(params: {
	text: string;
	node_type: string;
	disabled: boolean;

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
				disabled={params.disabled}
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
				disabled={params.disabled}
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
	getFlow: () => ReactFlowJsonObject<Node, Edge>;
	onChange: (select: components["schemas"]["PipelineInfo"] | null) => void;
}) {
	const { setNodes, setEdges, fitView } = useReactFlow();

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
		const res = serializePipeline(raw);

		if (res.result === "error") {
			console.error(`Could not serialize pipeline.`);
			console.error(res.message);
			return;
		}

		doSave.mutate(res.value);
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
						const de = deserializePipeline(params.pipeline.data);

						setNodes(de.nodes);
						setEdges(de.edges);
						fitView();
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

						{pipeline !== null ? null : (
							<Text ta="center" c="red" size="sm">
								Disabled. Select a pipeline before adding nodes
							</Text>
						)}

						<div className={style.node_group}>
							<div className={style.node_group_title}>Base</div>
							<AddNodeButton
								text="Input"
								node_type="pipelineinput"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="Constant"
								node_type="constant"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="IfNone"
								node_type="ifnone"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="Checksum"
								node_type="hash"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>
						</div>

						<div className={style.node_group}>
							<div className={style.node_group_title}>Storage</div>
							<AddNodeButton
								text="Add item"
								node_type="additem"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>
						</div>

						<div className={style.node_group}>
							<div className={style.node_group_title}>Audio</div>
							<AddNodeButton
								text="Strip tags"
								node_type="striptags"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>
							<AddNodeButton
								text="Extract tags"
								node_type="extracttags"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
							/>
							<AddNodeButton
								text="Extract covers"
								node_type="extractcovers"
								setNodes={setNodes}
								onInfo={() => {}}
								disabled={pipeline === null}
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
