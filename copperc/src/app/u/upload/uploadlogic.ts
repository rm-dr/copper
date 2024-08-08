/// Start an upload job on the server.

import { Dispatch, SetStateAction } from "react";
import { UploadQueuedFile, UploadState } from "./util";
import { APIclient } from "@/app/_util/api";

/// On success, calls `then()` with the new job id.
export async function startUploadJob(ac: AbortController): Promise<string> {
	const { data, error } = await APIclient.POST("/upload/new");

	if (error !== undefined) {
		throw error;
	}

	return data.job_id;
}

/// Start a new file upload for the given job.
/// On success, calls `then()` with the new file name.
///
/// This is used inside `uploadBlob`.
async function start_new_file(
	upload_job_id: string,
	file_name: string,
): Promise<string> {
	const { data, error } = await APIclient.POST("/upload/{job_id}/newfile", {
		body: {
			file_name,
		},
		params: {
			path: { job_id: upload_job_id },
		},
	});

	if (error !== undefined) {
		throw error;
	}

	return data.file_id;
}

// Upload the given `Blob`
export async function uploadBlob(params: {
	abort_controller: AbortController;
	upload_job_id: string;
	blob: Blob;
	file_name: string;
	max_fragment_size: number;
	onProgress: (total_uploaded_bytes: number) => void;
}): Promise<string> {
	const file_name = await start_new_file(
		params.upload_job_id,
		params.file_name,
	);

	var frag_count = 0;
	let uploaded_bytes = 0;

	const frag_hashes: string[] = [];

	for (
		let frag_idx = 0;
		frag_idx * params.max_fragment_size < params.blob.size;
		frag_idx += 1
	) {
		frag_count += 1;
		const byte_idx = frag_idx * params.max_fragment_size;

		/// Get the next fragment
		const last_byte = Math.min(
			byte_idx + params.max_fragment_size,
			params.blob.size,
		);

		const fragment = params.blob.slice(byte_idx, last_byte);

		const hash = await crypto.subtle
			.digest("SHA-256", await fragment.arrayBuffer())
			.then((h) => {
				let hex = [],
					view = new DataView(h);
				for (let i = 0; i < view.byteLength; i += 4)
					hex.push(("00000000" + view.getUint32(i).toString(16)).slice(-8));
				return hex.join("").toUpperCase();
			});

		frag_hashes.push(hash);

		const formData = new FormData();
		formData.append(
			"metadata",
			JSON.stringify({
				part_idx: frag_idx,
				part_hash: hash,
			}),
		);
		formData.append("fragment", fragment);

		const res = await fetch(
			`/api/upload/${params.upload_job_id}/${file_name}`,
			{
				method: "POST",
				body: formData,
				signal: params.abort_controller.signal,
			},
		);

		if (!res.ok) {
			throw Error(`Bad response from server: ${res.status}`);
		}

		uploaded_bytes += last_byte - byte_idx;
		params.onProgress(uploaded_bytes);
	}

	const final_hash = await crypto.subtle
		.digest("SHA-256", new TextEncoder().encode(frag_hashes.join("")))
		.then((h) => {
			let hex = [],
				view = new DataView(h);
			for (let i = 0; i < view.byteLength; i += 4)
				hex.push(("00000000" + view.getUint32(i).toString(16)).slice(-8));
			return hex.join("").toUpperCase();
		});

	let { data, error } = await APIclient.POST(
		"/upload/{job_id}/{file_id}/finish",
		{
			signal: params.abort_controller.signal,
			body: {
				frag_count,
				hash: final_hash,
			},
			params: {
				path: {
					job_id: params.upload_job_id,
					file_id: file_name,
				},
			},
		},
	);

	if (error !== undefined) {
		throw Error(error);
	}

	return file_name;
}

// Start uploading the given files.
//
// Returns an abortcontroller that cancels the upload,
// and a promise that resolves when all files have finished uploading.
export function startUploadingFiles(params: {
	setUploadState: Dispatch<SetStateAction<UploadState>>;
	onFinishFile: (upload_job_id: string, file_name: string) => void;
	files: UploadQueuedFile[];
}): [AbortController, Promise<any>] {
	// Abort controller for this set of uploads
	let upload_ac = new AbortController();

	// Remove a job from the queue
	// (failed)
	let fail_upload = (file: UploadQueuedFile) => {
		params.setUploadState((us) => {
			var new_queue = [...us.queue];
			for (var i = 0; i < new_queue.length; i++) {
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
		const request_body_limit = await APIclient.GET("/status").then(
			({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}

				return data.request_body_limit;
			},
		);

		for (const file of params.files) {
			let upload_job_id;
			try {
				upload_job_id = await startUploadJob(upload_ac);
			} catch (err: any) {
				if (err.name == "AbortError") {
					throw err;
				} else {
					fail_upload(file);
					continue;
				}
			}

			let file_name;
			try {
				file_name = await uploadBlob({
					abort_controller: upload_ac,
					upload_job_id,
					blob: file.file,
					file_name: file.file.name,

					// Leave a few KB for headers & metadata
					max_fragment_size: request_body_limit - 25_000,

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
			params.onFinishFile(upload_job_id, file_name);
			params.setUploadState((us) => {
				var new_queue = [...us.queue];
				for (var i = 0; i < new_queue.length; i++) {
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
	let promise = do_upload();

	promise = promise
		// When all jobs finish
		.then(() => {
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

	return [upload_ac, promise];
}
