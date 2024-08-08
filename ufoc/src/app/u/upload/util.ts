import { FileWithPath } from "@mantine/dropzone";

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

// Prettyprint a unit in bytes
export function ppBytes(bytes: number): string {
	let l = 0;

	while (bytes >= 1024 && ++l && l < 9) {
		bytes = bytes / 1024;
	}

	let unit = ["bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"][
		l
	];
	let number = bytes.toFixed(bytes < 10 && l > 0 ? 1 : 0);
	return `${number} ${unit}`;
}
