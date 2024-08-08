import { Badge, Button, Progress, Table, Text } from "@mantine/core";
import styles from "../page.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import { useEffect, useState } from "react";
import { UploadState } from "../util";
import { ppBytes } from "@/app/_util/ppbytes";
import {
	IconCpu,
	IconFileUpload,
	IconGridPattern,
	IconHexagonMinus,
	IconServer2,
	IconTrash,
	IconUpload,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";

type RunnerState = {
	queued_jobs: number;
	finished_jobs: number;
	failed_jobs: number;
	error: boolean;
};

export function useStatusPanel(params: {
	uploadState: UploadState;
	selectedPipeline: string | null;
	startUpload: () => void;
	clearQueue: () => void;
	stopUpload: () => void;
}) {
	const [runnerstate, setRunnerState] = useState<RunnerState>({
		queued_jobs: 0,
		finished_jobs: 0,
		failed_jobs: 0,
		error: false,
	});

	const size_of_queue = params.uploadState.queue.reduce(
		(sum, file) => sum + file.file.size - file.uploaded_bytes,
		0,
	);

	const size_unfinished = params.uploadState.queue.reduce(
		(sum, file) => sum + file.uploaded_bytes,
		0,
	);

	// Status auto-update
	useEffect(() => {
		const update_status = async () => {
			let res;
			try {
				res = await fetch("/api/status/runner");
			} catch {
				setRunnerState({
					queued_jobs: 0,
					finished_jobs: 0,
					failed_jobs: 0,
					error: true,
				});
				return;
			}

			if (!res.ok) {
				setRunnerState({
					queued_jobs: 0,
					finished_jobs: 0,
					failed_jobs: 0,
					error: true,
				});
				return;
			}

			let json;
			try {
				json = await res.json();
			} catch {
				setRunnerState({
					queued_jobs: 0,
					finished_jobs: 0,
					failed_jobs: 0,
					error: true,
				});
				return;
			}

			setRunnerState({
				queued_jobs: json.queued_jobs,
				finished_jobs: json.finished_jobs,
				failed_jobs: json.failed_jobs,
				error: false,
			});
		};

		update_status();
		const id = setInterval(update_status, 1000);
		return () => clearInterval(id);
	}, []);

	return (
		<>
			<Panel
				panel_id={styles.panel_id_status}
				icon={<XIcon icon={IconServer2} />}
				title={"System status"}
			>
				<div className={styles.status_panel_content}>
					<div>
						<PanelTitle
							icon={<XIcon icon={IconCpu} />}
							title={"Pipeline Jobs"}
							zeromargin
						/>
						<div className={styles.status_subpanel_content}>
							<JobTable
								pending_count={runnerstate.queued_jobs}
								complete_count={runnerstate.finished_jobs}
								failed_count={runnerstate.failed_jobs}
								error={runnerstate.error}
							/>
						</div>
					</div>

					<div>
						<PanelTitle
							icon={<XIcon icon={IconFileUpload} />}
							title={"Upload Jobs"}
							zeromargin
						/>
						<div className={styles.status_subpanel_content}>
							<JobTable
								pending_count={params.uploadState.queue.length}
								complete_count={params.uploadState.done_uploads}
								failed_count={params.uploadState.failed_uploads}
								error={false}
							/>
						</div>
					</div>

					<div>
						<PanelTitle
							icon={<XIcon icon={IconGridPattern} />}
							title={"Control panel"}
							zeromargin
						/>
						<div className={styles.status_subpanel_content}>
							<div
								style={{
									width: "90%",
									margin: "auto",
								}}
							>
								<Button.Group>
									<Button
										radius="0"
										disabled={
											params.uploadState.queue.length == 0 ||
											params.uploadState.is_uploading
										}
										onClick={params.clearQueue}
										variant="light"
										color="red"
										style={
											params.uploadState.queue.length == 0 ||
											params.uploadState.is_uploading
												? { cursor: "default" }
												: {}
										}
									>
										<XIcon icon={IconTrash} />
									</Button>
									<Button
										radius="0"
										loading={params.uploadState.is_uploading}
										disabled={
											params.uploadState.queue.length == 0 ||
											params.selectedPipeline === null
										}
										onClick={params.startUpload}
										variant="light"
										color="green"
										fullWidth
										leftSection={<XIcon icon={IconUpload} />}
										style={{ cursor: "default" }}
									>
										Upload {params.uploadState.queue.length} file
										{params.uploadState.queue.length == 1 ? "" : "s"}
									</Button>
									<Button
										radius="0"
										disabled={!params.uploadState.is_uploading}
										onClick={params.stopUpload}
										variant="light"
										color="red"
										style={
											!params.uploadState.is_uploading
												? { cursor: "default" }
												: {}
										}
									>
										<XIcon icon={IconHexagonMinus} />
									</Button>
								</Button.Group>
								<UploadBar
									disabled={params.uploadState.queue.length == 0}
									pending_size={size_of_queue}
									unfinished_size={size_unfinished}
									complete_size={params.uploadState.done_size}
									failed_size={params.uploadState.failed_size}
								/>
								<Text
									size="sm"
									c="dimmed"
									opacity={params.uploadState.is_uploading ? 1 : 0}
									style={{
										userSelect: "none",
										transition: "200ms",
									}}
								>
									Uploading files. Please stay on this page.
								</Text>
							</div>
						</div>
					</div>
				</div>
			</Panel>
		</>
	);
}

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
	var empty = 100;
	var pending = 0;
	var complete = 0;
	var failed = 0;
	var unfinished = 0;

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
			radius="0"
			style={{ opacity: disabled ? 0.5 : 1, transition: "100ms" }}
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

function JobTable({
	pending_count,
	complete_count,
	failed_count,
	error,
}: {
	pending_count: number;
	complete_count: number;
	failed_count: number;
	error: boolean;
}) {
	const total = pending_count + complete_count + failed_count;
	var err = 0; // Special segment for errors, allows pretty animation
	var pending = 100;
	var complete = 0;
	var failed = 0;

	if (error) {
		err = 100;
		pending = 0;
		failed = 0;
		complete = 0;
	} else if (total != 0) {
		pending = (pending_count / total) * 100;
		complete = (complete_count / total) * 100;
		failed = (failed_count / total) * 100;
	}

	return (
		<div style={{ userSelect: "none" }}>
			<Table>
				<Table.Tbody>
					<Table.Tr>
						<Table.Td>Pending</Table.Td>
						<Table.Td>
							<Badge color="gray">{error ? "?!" : pending_count}</Badge>
						</Table.Td>
					</Table.Tr>
					<Table.Tr>
						<Table.Td>Complete</Table.Td>
						<Table.Td>
							<Badge color="green">{error ? "?!" : complete_count}</Badge>
						</Table.Td>
					</Table.Tr>
					<Table.Tr>
						<Table.Td>Failed</Table.Td>
						<Table.Td>
							<Badge color="red">{error ? "?!" : failed_count}</Badge>
						</Table.Td>
					</Table.Tr>
				</Table.Tbody>
			</Table>
			<Progress.Root radius="0" transitionDuration={200}>
				<Progress.Section value={err} color="gray" animated />
				<Progress.Section value={failed} color="red" />
				<Progress.Section value={complete} color="green" />
				<Progress.Section value={pending} color="gray" />
			</Progress.Root>
		</div>
	);
}
