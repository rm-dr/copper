import { edgeclient } from "@/lib/api/client";
import { Dispatch, SetStateAction } from "react";
import { UploadQueuedFile, UploadState } from "./page";

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

/**
 * Upload the given files.
 *
 * This function starts a promise and returns an `AbortController` that may be used to
 * cancel it.
 */
export function uploadFiles(params: {
	setUploadState: Dispatch<SetStateAction<UploadState>>;
	onFinishFile: (upload_job_id: string) => void;
	abort_controller: AbortController;
	files: UploadQueuedFile[];
}): AbortController {
	// Abort controller for this set of uploads
	const upload_ac = new AbortController();

	// Remove a job from the queue
	// (failed)
	const fail_upload = (file: UploadQueuedFile) => {
		params.setUploadState((us) => {
			const new_queue = [...us.queue];
			for (let i = 0; i < new_queue.length; i++) {
				if ((new_queue[i] as UploadQueuedFile).uid === file.uid) {
					new_queue.splice(i, 1);
				} else {
					++i;
				}
			}

			return {
				...us,
				queue: new_queue,
				failed_uploads: (us.failed_uploads += 1),
				failed_size: (us.failed_size += file.file.size),
			};
		});
	};

	const do_upload = async () => {
		for (const file of params.files) {
			let upload_id;
			try {
				upload_id = await uploadBlob({
					abort_controller: upload_ac,
					blob: file.file,
					file_name: file.file.name,

					// Whenever we make progress on any file
					onProgress: (uploaded_bytes) => {
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
				});

				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} catch (err: any) {
				if (err.name == "AbortError") {
					throw err;
				} else {
					fail_upload(file);
					continue;
				}
			}

			// Remove this done job from the queue
			// (successful)
			params.onFinishFile(upload_id);
			params.setUploadState((us) => {
				const new_queue = [...us.queue];
				for (let i = 0; i < new_queue.length; i++) {
					if ((new_queue[i] as UploadQueuedFile).uid === file.uid) {
						new_queue.splice(i, 1);
					} else {
						++i;
					}
				}

				return {
					...us,
					queue: new_queue,
					done_size: us.done_size + file.file.size,
					done_uploads: us.done_uploads + 1,
				};
			});
		}
	};

	// Start uploading
	do_upload()
		.then(() => {
			// When all jobs finish

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

	return upload_ac;
}
