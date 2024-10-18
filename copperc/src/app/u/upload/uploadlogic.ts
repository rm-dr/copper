import { edgeclient } from "@/lib/api/client";
import { FileWithPath } from "@mantine/dropzone";

/**
 * Start a new upload job and return its id.
 */
export async function startUploadJob(mime: string): Promise<{
	job_id: string;
	request_body_limit: number;
}> {
	const { data, error } = await edgeclient.POST("/storage/upload", {
		body: {
			mime,
		},
	});

	if (error !== undefined) {
		throw error;
	}

	return {
		job_id: data.job_id,
		request_body_limit: data.request_body_limit,
	};
}

/**
 * Upload the provided blob.
 * Returns a promise that resolves when the upload is complete.
 */
export async function uploadBlob(params: {
	abort_controller: AbortController;
	blob: Blob;
	file_name: string;
	onProgress: (total_uploaded_bytes: number) => void;
}): Promise<string> {
	const upload_info = await startUploadJob("application/octet-stream");

	// Leave a few KB for headers & metadata
	const max_fragment_size = upload_info.request_body_limit - 16_000;

	// let frag_count = 0;
	// const frag_hashes: string[] = [];

	let uploaded_bytes = 0;

	for (
		let frag_idx = 0;
		frag_idx * max_fragment_size < params.blob.size;
		frag_idx += 1
	) {
		// frag_count += 1;
		const byte_idx = frag_idx * max_fragment_size;

		/// Get the next fragment
		const last_byte = Math.min(byte_idx + max_fragment_size, params.blob.size);

		const fragment = params.blob.slice(byte_idx, last_byte);

		/*
		const buffer = await crypto.subtle.digest(
			"SHA-256",
			await fragment.arrayBuffer(),
		);
		const hex = [];
		const view = new DataView(buffer);
		for (let i = 0; i < view.byteLength; i += 4) {
			hex.push(("00000000" + view.getUint32(i).toString(16)).slice(-8));
		}
		const hash = hex.join("").toUpperCase();
		frag_hashes.push(hash);
		*/

		const formData = new FormData();
		formData.append("part_data", fragment);

		const res = await edgeclient.POST("/storage/upload/{upload_id}/part", {
			params: { path: { upload_id: upload_info.job_id } },
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			body: formData as any,
			signal: params.abort_controller.signal,
		});

		if (res.response.status !== 200) {
			throw Error(
				`Bad response from server on upload: ${res.response.statusText}`,
			);
		}

		uploaded_bytes += last_byte - byte_idx;
		params.onProgress(uploaded_bytes);
	}

	/*
	const final_hash = await crypto.subtle
		.digest("SHA-256", new TextEncoder().encode(frag_hashes.join("")))
		.then((h) => {
			const hex = [];
			const view = new DataView(h);
			for (let i = 0; i < view.byteLength; i += 4)
				hex.push(("00000000" + view.getUint32(i).toString(16)).slice(-8));
			return hex.join("").toUpperCase();
		});
	*/

	const finish_res = await edgeclient.POST(
		"/storage/upload/{upload_id}/finish",
		{
			params: { path: { upload_id: upload_info.job_id } },
			signal: params.abort_controller.signal,
		},
	);

	if (finish_res.response.status !== 200) {
		throw Error(
			`Bad response from server on finish: ${finish_res.response.statusText}`,
		);
	}

	return upload_info.job_id;
}

export type QueuedFile = {
	uid: number;
	file: FileWithPath;
};

/**
 * Upload the given files.
 */
export async function uploadFiles(params: {
	files: QueuedFile[];
	abort_controller: AbortController;

	onProgress: (file: QueuedFile, uploaded_bytes: number) => void;
	onFailFile: (file: QueuedFile) => void;
	onFinishFile: (file: QueuedFile, upload_job_id: string) => void;
}) {
	const parallel_jobs = 3;

	function pickUpNextTask() {
		if (params.abort_controller.signal.aborted) {
			return null;
		}

		const file = params.files.shift();

		if (file !== undefined) {
			return (
				uploadBlob({
					abort_controller: params.abort_controller,
					blob: file.file,
					file_name: file.file.name,

					// Whenever we make progress on any file
					onProgress: (uploaded_bytes) => {
						params.onProgress(file, uploaded_bytes);
					},
				})
					.then((upload_id) => {
						params.onFinishFile(file, upload_id);
					})
					// eslint-disable-next-line @typescript-eslint/no-explicit-any
					.catch((err) => {
						if (err.name == "AbortError") {
							throw err;
						} else {
							params.onFailFile(file);
						}
					})
			);
		}

		return null;
	}

	function startChain() {
		return Promise.resolve().then(function next(): Promise<void> {
			const task = pickUpNextTask();
			if (task === null) {
				return Promise.resolve();
			} else {
				return task.then(next);
			}
		});
	}

	const jobs: Promise<void>[] = [];

	for (let i = 0; i < parallel_jobs; i++) {
		jobs.push(startChain());
	}

	await Promise.all(jobs);
}
