/**
 * Prettyprint a quantity of bytes
 *
 * @param bytes - A quantity of bytes
 * @returns - A pretty formatted string
 */
export function ppBytes(bytes: number): string {
	let l = 0;

	while (bytes >= 1024 && ++l && l < 9) {
		bytes = bytes / 1024;
	}

	const unit = [
		"bytes",
		"KiB",
		"MiB",
		"GiB",
		"TiB",
		"PiB",
		"EiB",
		"ZiB",
		"YiB",
	][l];

	const number = bytes.toFixed(bytes < 10 && l > 0 ? 1 : 0);

	return `${number} ${unit}`;
}
