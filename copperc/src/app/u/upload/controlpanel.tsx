"use client";

import TitleBar from "@/components/titlebar";
import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { Button, Progress, Select, Text } from "@mantine/core";
import { Dispatch, SetStateAction, useRef } from "react";
import { components } from "@/lib/api/openapi";

import styles from "./page.module.scss";
import { uploadFiles } from "./uploadlogic";
import { UploadState } from "./page";
import { ppBytes } from "@/lib/ppbytes";

function UploadBar({
	pending_size,
	complete_size,
	unfinished_size,
	failed_size,
	disabled,
}: {
	pending_size: number;
	complete_size: number;
	unfinished_size: number;
	failed_size: number;
	disabled: boolean;
}) {
	const total = pending_size + complete_size + failed_size + unfinished_size;
	// Use a special "empty" segment so that pending bar slides in and out
	let empty = 100;
	let pending = 0;
	let complete = 0;
	let failed = 0;
	let unfinished = 0;

	if (total != 0) {
		empty = 0;
		unfinished = (unfinished_size / total) * 100;
		pending = (pending_size / total) * 100;
		complete = (complete_size / total) * 100;
		failed = (failed_size / total) * 100;
	}

	return (
		<Progress.Root
			size="xl"
			style={{
				opacity: disabled ? 0.5 : 1,
				transition: "200ms",
				width: "100%",
			}}
			transitionDuration={200}
		>
			<Progress.Section value={failed} color="red">
				<Progress.Label>{ppBytes(failed_size)}</Progress.Label>
			</Progress.Section>
			<Progress.Section value={complete} color="green" animated>
				<Progress.Label>{ppBytes(complete_size)}</Progress.Label>
			</Progress.Section>
			<Progress.Section
				value={unfinished}
				color="green"
				style={{ opacity: 0.5 }}
				animated
			/>
			<Progress.Section value={pending} color="gray">
				<Progress.Label>{ppBytes(pending_size)}</Progress.Label>
			</Progress.Section>
			<Progress.Section value={empty} color="gray" />
		</Progress.Root>
	);
}

export function ControlPanel(params: {
	selected_pipeline: null | components["schemas"]["PipelineInfo"];
	uploadState: UploadState;

	setUploadState: Dispatch<SetStateAction<UploadState>>;
	setPipeline: (pipeline: null | components["schemas"]["PipelineInfo"]) => void;
}) {
	const upload_ac = useRef(new AbortController());

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

	const select_data =
		pipelines.data === undefined
			? []
			: pipelines.data.map((x) => {
					// This page can only start pipelines that take
					// exactly one `Blob` input.
					let disabled = false;
					const inputs = Object.entries(x.data.nodes).filter(
						(x) => x[1].node_type === "Input",
					);
					if (inputs.length !== 1) {
						disabled = true;
					} else {
						const data_type = inputs[0]![1].params?.input_type;
						if (data_type?.parameter_type !== "String") {
							disabled = true;
						} else {
							if (data_type.value !== "Blob") {
								disabled = true;
							}
						}
					}

					return {
						label: x.name,
						value: x.id.toString(),
						disabled,
					};
			  });

	const select_value =
		params.selected_pipeline === null
			? null
			: params.selected_pipeline.id.toString();

	const size_of_queue = params.uploadState.queue.reduce(
		(sum, file) => sum + file.file.size - file.uploaded_bytes,
		0,
	);

	const size_unfinished = params.uploadState.queue.reduce(
		(sum, file) => sum + file.uploaded_bytes,
		0,
	);

	return (
		<>
			<div className={styles.panel}>
				<TitleBar text="Control Panel" />
				<div className={styles.panel_content}>
					<Select
						label="Select pipeline"
						description={
							<>
								Select the pipeline to run with uploaded files.
								<br />
								Only pipelines with exactly one `Blob` input may be run from
								this page.
							</>
						}
						style={{ width: "100%" }}
						disabled={
							pipelines.data === undefined || params.uploadState.is_uploading
						}
						placeholder={
							pipelines.data === undefined ? "Loading..." : "Select a pipeline"
						}
						data={select_data}
						value={select_value}
						onChange={(value) => {
							const int = value === null ? null : parseInt(value);
							if (int === null || pipelines.data === undefined) {
								params.setPipeline(null);
								return;
							}

							const pipeline = pipelines.data.find((x) => x.id === int) || null;
							params.setPipeline(pipeline);
						}}
					/>

					<div
						style={{
							display: "flex",
							flexDirection: "column",
							gap: "0.2rem",
							width: "100%",
							marginTop: "1rem",
						}}
					>
						<UploadBar
							disabled={params.uploadState.queue.length == 0}
							pending_size={size_of_queue}
							unfinished_size={size_unfinished}
							complete_size={params.uploadState.done_size}
							failed_size={params.uploadState.failed_size}
						/>

						<Button.Group style={{ width: "100%" }}>
							<Button
								variant="light"
								fullWidth
								color="red"
								disabled={!params.uploadState.is_uploading}
								onClick={() => {
									upload_ac.current.abort();
								}}
							>
								Cancel
							</Button>
							<Button
								disabled={
									params.uploadState.queue.length === 0 ||
									params.selected_pipeline === null
								}
								loading={params.uploadState.is_uploading}
								variant="filled"
								fullWidth
								c="primary"
								onClick={() => {
									if (params.uploadState.is_uploading) {
										return;
									}

									params.setUploadState((us) => {
										return {
											...us,
											is_uploading: true,
										};
									});

									uploadFiles({
										abort_controller: upload_ac.current,
										files: params.uploadState.queue.map((x) => ({
											uid: x.uid,
											file: x.file,
										})),

										onProgress: (file, uploaded_bytes) => {
											params.setUploadState((us) => {
												for (const q of us.queue) {
													if (q.uid === file.uid) {
														q.uploaded_bytes = uploaded_bytes;
													}
												}

												return {
													...us,
												};
											});
										},

										onFailFile: (file) => {
											// Be careful with this, may cause weird behavior if not
											// implemented properly... Especially since we're
											// chaining promises in `uploadlogic`
											params.setUploadState((us) => ({
												...us,
												queue: us.queue.filter((x) => x.uid !== file.uid),
												failed_uploads: (us.failed_uploads += 1),
												failed_size: (us.failed_size += file.file.size),
											}));
										},

										onFinishFile: (file, upload_job) => {
											// Be careful with this, may cause weird behavior if not
											// implemented properly... Especially since we're
											// chaining promises in `uploadlogic`
											params.setUploadState((us) => ({
												...us,
												queue: us.queue.filter((x) => x.uid !== file.uid),
												done_size: us.done_size + file.file.size,
												done_uploads: us.done_uploads + 1,
											}));

											if (params.selected_pipeline === null) {
												return;
											}

											// This should have exactly input, and it's type should be `Blob`.
											// Find its name.
											let input_name: string | null = null;
											const inputs = Object.entries(
												params.selected_pipeline.data.nodes,
											).filter((x) => x[1].node_type === "Input");
											if (inputs.length === 1) {
												const input = inputs[0]![1];
												const p_input_type = input.params?.input_type;
												const p_input_name = input.params?.input_name;
												if (
													p_input_type?.parameter_type === "String" &&
													p_input_type.value === "Blob" &&
													p_input_name?.parameter_type === "String"
												) {
													input_name = p_input_name.value;
												}
											}

											if (input_name === null) {
												console.error("input_name is null, not starting job");
												// Return early if this isn't a valid pipeline
												// (this shouldn't be possible, invalid pipelines
												// are disabled in the <Select/> above.)
												return;
											}

											edgeclient.POST("/pipeline/{pipeline_id}/run", {
												params: {
													path: { pipeline_id: params.selected_pipeline.id },
												},
												body: {
													job_id: `${params.selected_pipeline.name}-${upload_job}`,
													input: {
														[input_name]: {
															type: "Blob",
															upload_id: upload_job,
														},
													},
												},
											});
										},
									})
										.then(() => {
											// Refresh abort controller,
											// the previous one may have been cancelled
											upload_ac.current = new AbortController();

											params.setUploadState((us) => {
												// Notice we don't clear the queue,
												// since some queued jobs may not have been started.
												// (this will happen if files are dragged in while we're uploading)
												return {
													...us,
													done_size: 0,
													failed_size: 0,
													is_uploading: false,
												};
											});
										})
										// Aborted by user
										.catch((err) => {
											// Refresh abort controller,
											// the previous one may have been cancelled
											upload_ac.current = new AbortController();

											if (err.name != "AbortError") {
												throw err;
											}

											params.setUploadState((us) => {
												// Clean up partially-completed jobs
												for (const q of us.queue) {
													q.uploaded_bytes = 0;
												}

												return {
													...us,
													is_uploading: false,
												};
											});
										})
										// Other errors
										.catch((err) => {
											console.log(err);
										});
								}}
							>
								Upload queued files
							</Button>
						</Button.Group>

						<Text ta="center" c="dimmed">
							{params.selected_pipeline === null
								? "We cannot upload files without a pipeline to run."
								: params.uploadState.queue.length === 0
								? "Cannot upload, the queue is empty."
								: params.uploadState.is_uploading
								? "Uploading..."
								: null}
						</Text>
					</div>
				</div>
			</div>
		</>
	);
}
