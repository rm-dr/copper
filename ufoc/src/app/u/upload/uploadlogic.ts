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
	file_extension: string,
): Promise<string> {
	const { data, error } = await APIclient.POST("/upload/{job_id}/newfile", {
		body: {
			file_extension,
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
	file_extension: string;
	max_fragment_size: number;
	onProgress: (total_uploaded_bytes: number) => void;
}): Promise<string> {
	const file_name = await start_new_file(
		params.upload_job_id,
		params.file_extension,
	);

	var frag_count = 0;
	let uploaded_bytes = 0;

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

		const formData = new FormData();
		formData.append(
			"metadata",
			JSON.stringify({
				part_idx: frag_idx,
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

	let res = await fetch(
		`/api/upload/${params.upload_job_id}/${file_name}/finish`,
		{
			signal: params.abort_controller.signal,
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				frag_count,
				hash: "TODO",
			}),
		},
	);

	if (!res.ok) {
		throw Error(`Bad response from server: ${res.status}`);
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
				var c = file.file.name.split(".");
				c.shift();
				file_name = await uploadBlob({
					abort_controller: upload_ac,
					upload_job_id,
					blob: file.file,
					file_extension: c.join("."),
					max_fragment_size: 1500000, // TODO: get from server

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
