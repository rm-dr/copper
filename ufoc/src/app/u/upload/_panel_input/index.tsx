import { Group, Progress, Text } from "@mantine/core";
import { Dropzone } from "@mantine/dropzone";
import { Panel, PanelTitle } from "@/app/components/panel";
import { Dispatch, SetStateAction } from "react";
import styles from "../page.module.scss";
import { UploadQueuedFile, UploadState, ppBytes } from "../util";
import {
	XIconFile,
	XIconFilePlus,
	XIconFileX,
	XIconList,
	XIconPlus,
	XIconSend,
	XIconX,
} from "@/app/components/icons";

/*
const updateScrollFade = () => {
		const e = document.getElementById(styles.filelist_base);

		if (e == null) {
			return;
		}

		const isScrollable = e.scrollHeight > e.clientHeight;

		if (!isScrollable) {
			e.classList.remove(styles.fade_top, styles.fade_bot);
			return;
		}

		const isScrolledToBottom =
			e.scrollHeight < e.clientHeight + e.scrollTop + 1;

		const isScrolledToTop = isScrolledToBottom ? false : e.scrollTop === 0;

		e.classList.toggle(styles.fade_bot, !isScrolledToBottom);
		e.classList.toggle(styles.fade_top, !isScrolledToTop);
	};
*/

export function useInputPanel({
	uploadState,
	setUploadState,
}: {
	uploadState: UploadState;
	setUploadState: Dispatch<SetStateAction<UploadState>>;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_id_input}
				icon={<XIconSend />}
				title={"Input"}
			>
				<PanelTitle icon={<XIconPlus />} title={"Select files"} />
				<Dropzone
					onDrop={(dropped_files) => {
						// Only add new files
						// TODO: improve this check
						const d = dropped_files.filter((file) => {
							let is_already_there = false;
							for (var i = 0; i < uploadState.queue.length; i++) {
								var f = uploadState.queue[i];
								if (f.file.path == file.path) {
									is_already_there = true;
								}
								if (is_already_there) {
									break;
								}
							}
							return !is_already_there;
						});
						setUploadState((us) => {
							let new_files = d.map((file) => {
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
					}}
					preventDropOnDocument={true}
					disabled={uploadState.is_uploading}
				>
					<Group justify="center" gap="xl" style={{ pointerEvents: "none" }}>
						<div
							style={{
								marginTop: "2rem",
								marginBottom: "2rem",
							}}
						>
							<Dropzone.Accept>
								<XIconFilePlus
									style={{
										height: "7rem",
										color: "var(--mantine-color-green-6)",
									}}
								/>
								<Text size="lg" inline c="green">
									Add files to upload queue
								</Text>
							</Dropzone.Accept>
							<Dropzone.Reject>
								<XIconX
									style={{
										height: "7rem",
										color: "var(--mantine-color-red-6)",
									}}
								/>
								<Text size="lg" inline c="red">
									Cannot add these files
								</Text>
							</Dropzone.Reject>
							<Dropzone.Idle>
								{uploadState.is_uploading ? (
									<>
										<XIconFileX
											style={{
												height: "7rem",
												color: "var(--mantine-color-dimmed)",
											}}
										/>
										<Text size="lg" inline c="dimmed">
											Files cannot be added while uploading
										</Text>
									</>
								) : (
									<>
										<XIconFile
											style={{
												height: "7rem",
											}}
										/>
										<Text size="lg" inline>
											Drag files here or click to open dialog
										</Text>
									</>
								)}
							</Dropzone.Idle>
						</div>
					</Group>
				</Dropzone>

				<PanelTitle icon={<XIconList />} title={"File list"} />
				<div id={styles.filelist_base}>
					{uploadState.queue.slice(0, 30).map((f, idx) => {
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
				<div className={styles.filelist_footer_container}>
					<div className={styles.filelist_footer}>
						{uploadState.queue.length} pending upload
						{uploadState.queue.length == 1 ? "" : "s"}
					</div>
				</div>
			</Panel>
		</>
	);
}

const FilelistEntry = ({
	file,
	progress,
	progress_label,
}: {
	file: UploadQueuedFile;
	progress: number;
	progress_label: string;
}) => {
	return (
		<div key={file.file.name} className={styles.filelist_entry_container}>
			<div key={file.file.name} className={styles.filelist_entry}>
				<div className={styles.filelist_filename}>
					<Text inherit truncate="end">
						{file.file.name}
					</Text>
				</div>
				<div className={styles.filelist_uploadprogress}>
					<Progress.Root size="xl" transitionDuration={200}>
						<Progress.Section value={progress} color="red" animated>
							<Progress.Label>{progress.toFixed(1)}%</Progress.Label>
						</Progress.Section>
						<Progress.Section value={100 - progress} color="gray">
							<Progress.Label>{progress_label}</Progress.Label>
						</Progress.Section>
					</Progress.Root>
				</div>
			</div>
		</div>
	);
};
