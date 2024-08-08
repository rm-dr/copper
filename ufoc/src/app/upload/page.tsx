"use client";

import { useRef, useState } from "react";

import styles from "./page.module.scss";
import { startUploadingFiles } from "./uploadlogic";
import { usePipelinePanel } from "./_panel_pipeline";
import { useStatusPanel } from "./_panel_status";
import { useInputPanel } from "./_panel_input";
import { UploadState } from "./util";

export default function Page() {
	const [uploadState, setUploadState] = useState<UploadState>({
		queue: [],
		done_size: 0,
		done_uploads: 0,
		failed_size: 0,
		failed_uploads: 0,
		file_id_counter: 0,
		is_uploading: false,
	});

	const [selectedPipeline, setSelectedPipeline] = useState<string | null>(null);
	const [selectedDataset, setSelectedDataset] = useState<string | null>(null);

	var upload_ac = useRef(new AbortController());

	const panel_pipeline = usePipelinePanel({
		setSelectedPipeline,
		setSelectedDataset,
		selectedDataset,
	});

	const panel_input = useInputPanel({
		uploadState,
		setUploadState,
	});

	const panel_status = useStatusPanel({
		uploadState: uploadState,
		selectedPipeline,

		stopUpload: () => {
			upload_ac.current.abort();
		},

		clearQueue: () => {
			setUploadState((us) => {
				return {
					...us,
					done_size: 0,
					queue: [],
				};
			});
		},

		startUpload: () => {
			if (uploadState.is_uploading) {
				return;
			}

			setUploadState((us) => {
				return {
					...us,
					is_uploading: true,
				};
			});

			const [ac, _promise] = startUploadingFiles({
				setUploadState,
				onFinishFile: (upload_job, file_name) => {
					fetch(`/api/pipelines/${selectedPipeline}/run`, {
						method: "POST",
						headers: {
							"Content-Type": "application/json",
						},
						body: JSON.stringify({
							input: {
								type: "File",
								file_name,
								upload_job,
							},
						}),
					});
				},
				files: uploadState.queue,
			});

			// Refresh abort controller,
			// the previous one may have been cancelled
			upload_ac.current = ac;
		},
	});

	return (
		<main className={styles.main}>
			{panel_status}
			{panel_pipeline}
			{panel_input}
		</main>
	);
}
