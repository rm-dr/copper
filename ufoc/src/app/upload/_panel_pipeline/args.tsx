import { Switch, TextInput } from "@mantine/core";
import { useState } from "react";
import styles from "../page.module.scss";

export function useArgBoolean(
	name: string,
	onChange: (value: boolean) => void,
) {
	// This lets us toggle the switch by clicking its container
	const [checked, setChecked] = useState(false);

	return (
		<div className={styles.arg}>
			<div
				className={styles.argleft}
				onMouseDown={() => {
					onChange(!checked);
					setChecked(!checked);
				}}
			>
				<div className={styles.argname}>{name}</div>
			</div>
			<div className={styles.argright}>
				<Switch
					checked={checked}
					onChange={(event) => {
						setChecked(event.currentTarget.checked);
						onChange(event.currentTarget.checked);
					}}
				/>
			</div>
		</div>
	);
}

export function useArgText(name: string, onChange: (value: string) => void) {
	return (
		<div className={styles.arg}>
			<div className={styles.argleft}>
				<div className={styles.argname}>Genre</div>
			</div>
			<div className={styles.argright}>
				<TextInput
					placeholder="Genre..."
					size="xs"
					onChange={(event) => {
						onChange(event.currentTarget.value);
					}}
				/>
			</div>
		</div>
	);
}
