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
