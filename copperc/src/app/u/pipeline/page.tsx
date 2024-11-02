"use client";

import style from "./pipeline.module.scss";
import "@xyflow/react/dist/style.css";

import React, { useCallback, useState } from "react";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { components } from "@/lib/api/openapi";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { Button, Select, Text } from "@mantine/core";

import { useAddPipelineModal } from "./_modals/addpipeline";
import { useFlow } from "./flow";
import { deserializePipeline } from "./serde";
import {
	AddNodeButton,
	PipelineDeleteButton,
	PipelineReloadButton,
	PipelineRenameButton,
	PipelineSaveButton,
} from "./buttons";
import { NavBlocker } from "@/components/navblock";

function Main() {
	const [isModified, setModified] = useState<boolean>(false);
	const [isReloading, setReloading] = useState<boolean>(false);
	const [isSaving, setSaving] = useState<boolean>(false);
	const { setNodes, setEdges, fitView } = useReactFlow();
	const { flow, getFlow } = useFlow({
		disabled: isSaving || isReloading,
		onModify: () => {
			setModified(true);
		},
	});

	const qc = useQueryClient();

	const [pipeline, _setPipeline] = useState<
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

	const setPipeline = useCallback(
		async (
			new_pipeline: components["schemas"]["PipelineInfo"] | null,
			fit?: boolean,
		) => {
			setReloading(true);
			if (new_pipeline === null) {
				setNodes([]);
				setEdges([]);
			} else {
				const de = await deserializePipeline(new_pipeline.data);
				setNodes(de.nodes);
				setEdges(de.edges);
			}

			_setPipeline(new_pipeline);

			// Hack that makes sure `fitView` is called _after_ nodes are updated.
			// This also gives us a minimum of 500ms while reloading time
			// so that the user sees a loader.
			setTimeout(() => {
				if (fit) {
					fitView();
				}

				setModified(false);
				setReloading(false);
			}, 500);
		},
		[fitView, setEdges, setNodes],
	);

	const { open: openAddPipeline, modal: modalAddPipeline } =
		useAddPipelineModal({
			onSuccess: (new_info) => {
				setPipeline(new_info).then(() => {
					qc.invalidateQueries({ queryKey: ["dataset/list"] });
					pipelines.refetch();
				});
			},
		});

	return (
		<>
			{modalAddPipeline}
			{isModified || isSaving ? <NavBlocker /> : null}

			<div className={style.pipeline_container}>
				<div className={style.tools_container}>
					<div className={style.tools_section}>
						<div className={style.tools_section_title}>Select pipeline</div>

						<Button
							fullWidth
							variant="subtle"
							size="xs"
							onClick={openAddPipeline}
							disabled={isModified || isReloading || isSaving}
						>
							New pipeline
						</Button>

						<Select
							disabled={
								pipelines.data === undefined ||
								isModified ||
								isReloading ||
								isSaving
							}
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
									return;
								}

								const new_pipeline =
									pipelines.data?.find((x) => x.id === int) || null;

								setPipeline(new_pipeline);
							}}
						/>

						{pipeline === null ? null : (
							<>
								<Button.Group style={{ width: "100%" }}>
									<PipelineDeleteButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={isModified || isReloading || isSaving}
										onSuccess={() => {
											qc.invalidateQueries({ queryKey: ["dataset/list"] });
											pipelines.refetch();
											setPipeline(null);
										}}
									/>

									<PipelineRenameButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={isModified || isReloading || isSaving}
										onSuccess={(select) => {
											qc.invalidateQueries({ queryKey: ["dataset/list"] });
											pipelines.refetch();
											setPipeline(select);
										}}
									/>
								</Button.Group>

								{!isModified ? null : (
									<>
										<Text ta="center" c="dimmed" size="xs">
											Pipeline has been modified. Save or reload to rename,
											delete, or select another pipeline.
										</Text>
									</>
								)}

								<Button.Group style={{ width: "100%" }}>
									<PipelineReloadButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={!isModified || isSaving}
										reloading={isReloading}
										onClick={() => {
											setReloading(true);
											qc.invalidateQueries({ queryKey: ["dataset/list"] }).then(
												() => {
													pipelines.refetch().then(() => {
														setPipeline(pipeline, false);
													});
												},
											);
										}}
									/>

									<PipelineSaveButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={!isModified || isReloading}
										onStart={() => setSaving(true)}
										onSuccess={(new_pipeline) => {
											setReloading(true);
											qc.invalidateQueries({ queryKey: ["dataset/list"] }).then(
												() => {
													pipelines.refetch().then(() => {
														setPipeline(new_pipeline, false).then(() => {
															setSaving(false);
														});
													});
												},
											);
										}}
									/>
								</Button.Group>

								{isModified ? null : (
									<Text ta="center" c="dimmed" size="xs">
										Pipeline has not been modified.
									</Text>
								)}
							</>
						)}
					</div>

					<div className={style.tools_section}>
						<div className={style.tools_section_title}>Add nodes</div>

						{pipeline !== null ? null : (
							<Text ta="center" c="dimmed" size="sm">
								No pipeline selected. Select a pipeline before adding nodes.
							</Text>
						)}

						<div className={style.node_group}>
							<div className={style.node_group_title}>Base</div>
							<AddNodeButton
								text="Input"
								node_type="pipelineinput"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>

							<AddNodeButton
								text="Constant"
								node_type="constant"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>

							<AddNodeButton
								text="IfNone"
								node_type="ifnone"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>

							<AddNodeButton
								text="Checksum"
								node_type="hash"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>
						</div>

						<div className={style.node_group}>
							<div className={style.node_group_title}>Storage</div>
							<AddNodeButton
								text="Add item"
								node_type="additem"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>
						</div>

						<div className={style.node_group}>
							<div className={style.node_group_title}>Audio</div>
							<AddNodeButton
								text="Strip tags"
								node_type="striptags"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>
							<AddNodeButton
								text="Extract tags"
								node_type="extracttags"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
							/>
							<AddNodeButton
								text="Extract covers"
								node_type="extractcovers"
								setNodes={setNodes}
								onInfo={() => {
									console.log("todo");
								}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null || isReloading || isSaving}
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
