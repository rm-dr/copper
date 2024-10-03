import style from "./pipeline.module.scss";
import nodestyle from "./_nodes/nodes.module.scss";

import { components } from "@/lib/api/openapi";
import { ActionIcon, Button } from "@mantine/core";
import { Edge, Node, ReactFlowJsonObject } from "@xyflow/react";
import { useDeletePipelineModal } from "./_modals/deletepipeline";
import { useRenamePipelineModal } from "./_modals/renamepipeline";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { Dispatch, SetStateAction, useCallback } from "react";
import { serializePipeline } from "./serde";
import { nodeDefinitions } from "./_nodes";
import { InfoIcon } from "lucide-react";

export function PipelineDeleteButton(params: {
	pipeline: components["schemas"]["PipelineInfo"];
	disabled: boolean;
	getFlow: () => ReactFlowJsonObject<Node, Edge>;
	onSuccess: () => void;
}) {
	const { open: openDeletePipeline, modal: modalDeletePipeline } =
		useDeletePipelineModal({
			pipeline_id: params.pipeline.id,
			pipeline_name: params.pipeline.name,
			onSuccess: params.onSuccess,
		});

	return (
		<>
			{modalDeletePipeline}
			<Button
				fullWidth
				variant="subtle"
				size="xs"
				onClick={openDeletePipeline}
				disabled={params.disabled}
			>
				Delete
			</Button>
		</>
	);
}

export function PipelineRenameButton(params: {
	pipeline: components["schemas"]["PipelineInfo"];
	disabled: boolean;
	getFlow: () => ReactFlowJsonObject<Node, Edge>;
	onSuccess: (select: components["schemas"]["PipelineInfo"] | null) => void;
}) {
	const { open: openRenamePipeline, modal: modalRenamePipeline } =
		useRenamePipelineModal({
			pipeline_id: params.pipeline.id,
			pipeline_name: params.pipeline.name,
			onSuccess: params.onSuccess,
		});

	return (
		<>
			{modalRenamePipeline}
			<Button
				fullWidth
				variant="subtle"
				size="xs"
				onClick={openRenamePipeline}
				disabled={params.disabled}
			>
				Rename
			</Button>
		</>
	);
}

export function PipelineReloadButton(params: {
	pipeline: components["schemas"]["PipelineInfo"];
	disabled: boolean;
	reloading: boolean;
	getFlow: () => ReactFlowJsonObject<Node, Edge>;
	onClick: () => void;
}) {
	return (
		<>
			<Button
				fullWidth
				variant="subtle"
				size="xs"
				disabled={params.disabled}
				loading={params.reloading}
				onClick={params.onClick}
			>
				Reload
			</Button>
		</>
	);
}

export function PipelineSaveButton(params: {
	pipeline: components["schemas"]["PipelineInfo"];
	disabled: boolean;
	getFlow: () => ReactFlowJsonObject<Node, Edge>;
	onStart: () => void;
	onSuccess: (select: components["schemas"]["PipelineInfo"] | null) => void;
}) {
	const doSave = useMutation({
		mutationFn: async (new_data: components["schemas"]["PipelineJson"]) => {
			return (
				await Promise.all([
					edgeclient.PATCH("/pipeline/{pipeline_id}", {
						params: { path: { pipeline_id: params.pipeline.id } },
						body: { new_data },
					}),

					// Minimum wait time, so we get a visible loader
					new Promise((resolve) => setTimeout(resolve, 500)),
				])
			)[0];
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				params.onSuccess(res.data!);
			} else {
				throw new Error(res.error);
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const savePipeline = useCallback(() => {
		params.onStart();

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
			<Button
				fullWidth
				variant="subtle"
				size="xs"
				disabled={params.disabled}
				onClick={savePipeline}
				loading={doSave.isPending}
			>
				Save
			</Button>
		</>
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

export function AddNodeButton(params: {
	text: string;
	node_type: string;
	disabled: boolean;

	setNodes: Dispatch<SetStateAction<Node[]>>;
	onInfo: () => void;
	onModify: () => void;
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
					params.onModify();
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
