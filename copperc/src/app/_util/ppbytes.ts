// Prettyprint a quantity of bytes
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
