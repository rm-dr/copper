/// Start an upload job on the server.

import { Dispatch, SetStateAction } from "react";
import { UploadQueuedFile, UploadState } from "./util";

/// On success, calls `then()` with the new job id.
export async function startUploadJob(ac: AbortController): Promise<string> {
	const res = await fetch("/api/upload/new", {
		method: "POST",
	});

	if (res.ok) {
		const j = await res.json();
		return j.job_id;
	} else {
		throw {
			status: res.status,
			text: await res.text(),
		};
	}
}

/// Start a new file upload for the given job.
/// On success, calls `then()` with the new file name.
///
/// This is used inside `uploadBlob`.
async function start_new_file(upload_job_id: string): Promise<string> {
	const res = await fetch(`/api/upload/${upload_job_id}/newfile`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			// TODO: fix
			file_type: "Blob",
		}),
	});

	if (res.ok) {
		const j = await res.json();
		return j.file_name;
	} else {
		throw {
			status: res.status,
			text: await res.text(),
		};
	}
}

// Upload the given `Blob`
export async function uploadBlob(params: {
	abort_controller: AbortController;
	upload_job_id: string;
	blob: Blob;
	max_fragment_size: number;
	onProgress: (total_uploaded_bytes: number) => void;
}): Promise<string> {
	const file_name = await start_new_file(params.upload_job_id);

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

		//TODO: handle res errors
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
				if (new_queue[i].uid === file.uid) {
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
		for (let i = 0; i < params.files.length; i++) {
			let file = params.files[i];
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
					max_fragment_size: 1500000, // TODO: get from server

					// Whenever we make progress on any file
					onProgress: (uploaded_bytes) => {
						params.setUploadState((us) => {
							for (var i = 0; i < us.queue.length; i++) {
								if (us.queue[i].uid === file.uid) {
									us.queue[i].uploaded_bytes = uploaded_bytes;
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
					if (new_queue[i].uid === file.uid) {
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
				for (var i = 0; i < us.queue.length; i++) {
					us.queue[i].uploaded_bytes = 0;
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
