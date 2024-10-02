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

function Main() {
	const [isModified, setModified] = useState<boolean>(false);
	const { setNodes, setEdges, fitView } = useReactFlow();
	const { flow, getFlow } = useFlow({
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
		(pipeline: components["schemas"]["PipelineInfo"] | null, fit?: boolean) => {
			if (pipeline === null) {
				setNodes([]);
				setEdges([]);
			} else {
				const de = deserializePipeline(pipeline.data);
				setNodes(de.nodes);
				setEdges(de.edges);
			}

			_setPipeline(pipeline);

			// Hack that makes sure `fitView` is called _after_ nodes are updated
			setTimeout(() => {
				if (fit) {
					fitView();
				}

				setModified(false);
			}, 100);
		},
		[fitView, setEdges, setNodes],
	);

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
							disabled={isModified}
						>
							New pipeline
						</Button>

						<Select
							disabled={pipelines.data === undefined || isModified}
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
								const pipeline =
									pipelines.data?.find((x) => x.id === int) || null;

								setPipeline(pipeline);
							}}
						/>

						{pipeline === null ? null : (
							<>
								<Button.Group style={{ width: "100%" }}>
									<PipelineDeleteButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={isModified}
										onSuccess={() => {
											qc.invalidateQueries({ queryKey: ["dataset/list"] });
											pipelines.refetch();
											setPipeline(null);
										}}
									/>

									<PipelineRenameButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={isModified}
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
										disabled={!isModified}
										onClick={() => {
											qc.invalidateQueries({ queryKey: ["dataset/list"] });
											pipelines.refetch();
											setPipeline(pipeline);
										}}
									/>

									<PipelineSaveButton
										pipeline={pipeline}
										getFlow={getFlow}
										disabled={!isModified}
										onSuccess={(new_pipeline) => {
											qc.invalidateQueries({ queryKey: ["dataset/list"] });
											pipelines.refetch();
											setPipeline(new_pipeline, false);
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
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="Constant"
								node_type="constant"
								setNodes={setNodes}
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="IfNone"
								node_type="ifnone"
								setNodes={setNodes}
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null}
							/>

							<AddNodeButton
								text="Checksum"
								node_type="hash"
								setNodes={setNodes}
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
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
								onModify={() => {
									setModified(true);
								}}
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
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null}
							/>
							<AddNodeButton
								text="Extract tags"
								node_type="extracttags"
								setNodes={setNodes}
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
								disabled={pipeline === null}
							/>
							<AddNodeButton
								text="Extract covers"
								node_type="extractcovers"
								setNodes={setNodes}
								onInfo={() => {}}
								onModify={() => {
									setModified(true);
								}}
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
