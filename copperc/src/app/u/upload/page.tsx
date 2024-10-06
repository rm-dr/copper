"use client";

import TitleBar from "@/components/titlebar";
import { Group, Progress, Text } from "@mantine/core";
import { useState } from "react";
import { components } from "@/lib/api/openapi";
import { Dropzone, FileWithPath } from "@mantine/dropzone";
import { FileUp, FileX } from "lucide-react";
import { ppBytes } from "@/lib/ppbytes";

import styles from "./page.module.scss";
import { ControlPanel } from "./controlpanel";

export type UploadQueuedFile = {
	uid: number;
	file: FileWithPath;
	uploaded_bytes: number;
};

export type UploadState = {
	// Number of completed uploads
	done_uploads: number;
	// Size of completed uploads, in bytes.
	// Only updated when a job completely finishes.
	done_size: number;

	failed_uploads: number;
	failed_size: number;

	// True if we're uploading right now
	is_uploading: boolean;

	file_id_counter: number;
	queue: UploadQueuedFile[];
};

export default function Page() {
	const [pipeline, setPipeline] = useState<
		components["schemas"]["PipelineInfo"] | null
	>(null);

	const [uploadState, setUploadState] = useState<UploadState>({
		queue: [],
		done_size: 0,
		done_uploads: 0,
		failed_size: 0,
		failed_uploads: 0,
		file_id_counter: 0,
		is_uploading: false,
	});

	return (
		<>
			<div className={styles.main}>
				<ControlPanel
					uploadState={uploadState}
					selected_pipeline={pipeline}
					setUploadState={setUploadState}
					setPipeline={setPipeline}
				/>

				<div className={styles.panel}>
					<TitleBar text="Add files" />
					<div className={styles.panel_content}>
						<Dropzone
							className={styles.dropzone}
							acceptColor="green"
							onDrop={(dropped_files) => {
								// Only add new files
								const d = dropped_files.filter((file) => {
									let is_already_there = false;
									for (let i = 0; i < uploadState.queue.length; i++) {
										const f = uploadState.queue[i]!;
										if (f.file.path === file.path) {
											is_already_there = true;
										}
										if (is_already_there) {
											break;
										}
									}
									return !is_already_there;
								});
								setUploadState((us) => {
									const new_files = d.map((file) => {
										return {
											file,
											uid: us.file_id_counter++,
											uploaded_bytes: 0,
										} as UploadQueuedFile;
									});

									return {
										...us,
										queue: [...us.queue, ...new_files],
									};
								});

								console.log(uploadState);
							}}
							preventDropOnDocument={true}
							disabled={uploadState.is_uploading}
						>
							<Group
								justify="center"
								gap="xl"
								style={{ pointerEvents: "none" }}
							>
								<Dropzone.Accept>
									<div className={styles.dropzone_inner}>
										<FileUp size="3rem" color="var(--mantine-color-green-5)" />
										<Text size="lg" inline c="green">
											Add files to upload queue
										</Text>
									</div>
								</Dropzone.Accept>

								<Dropzone.Reject>
									<div className={styles.dropzone_inner}>
										<FileX size="3rem" color="var(--mantine-color-red-5)" />
										<Text size="lg" inline c="red">
											Cannot add these files
										</Text>
									</div>
								</Dropzone.Reject>

								<Dropzone.Idle>
									<div className={styles.dropzone_inner}>
										{uploadState.is_uploading ? (
											<>
												<FileX
													size="3rem"
													color="var(--mantine-color-dimmed)"
												/>
												<Text size="lg" inline c="dimmed">
													Files cannot be added while uploading
												</Text>
											</>
										) : (
											<>
												<FileUp size="3rem" />
												<Text size="lg" inline>
													Drag files here or click to open dialog
												</Text>
											</>
										)}
									</div>
								</Dropzone.Idle>
							</Group>
						</Dropzone>

						<TitleBar
							text={`${uploadState.queue.length} pending upload${
								uploadState.queue.length == 1 ? "" : "s"
							}`}
						/>

						<div className={styles.filelist_scroll}>
							{uploadState.queue.slice(0, 30).map((f) => {
								return (
									<FilelistEntry
										key={f.uid}
										file={f}
										progress={(f.uploaded_bytes / f.file.size) * 100}
										progress_label={ppBytes(f.file.size - f.uploaded_bytes)}
									/>
								);
							})}
						</div>
					</div>
				</div>
			</div>
		</>
	);
}

function FilelistEntry({
	file,
	progress,
	progress_label,
}: {
	file: UploadQueuedFile;
	progress: number;
	progress_label: string;
}) {
	return (
		<div className={styles.filelist_entry}>
			<div className={styles.entry_progress}>
				<Progress.Root size="xl" transitionDuration={200}>
					<Progress.Section value={progress} c="primary" animated>
						<Progress.Label>{progress.toFixed(1)}%</Progress.Label>
					</Progress.Section>
					<Progress.Section value={100 - progress} color="gray">
						<Progress.Label>{progress_label}</Progress.Label>
					</Progress.Section>
				</Progress.Root>
			</div>

			<div className={styles.entry_text}>{file.file.name}</div>
		</div>
	);
}
