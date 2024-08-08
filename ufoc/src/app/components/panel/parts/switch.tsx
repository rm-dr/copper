import { Switch } from "@mantine/core";
import { useState } from "react";
import styles from "./parts.module.scss";

export function PanelSwitch(params: {
	name?: string;
	onChange?: (value: boolean) => void;
}) {
	// This lets us toggle the switch by clicking its container
	const [checked, setChecked] = useState(false);

	return (
		<div className={styles.panelpart}>
			<div
				className={styles.label}
				onMouseDown={(e) => {
					if (e.button === 0) {
						if (params.onChange !== undefined) {
							params.onChange(!checked);
						}
						setChecked(!checked);
					}
				}}
			>
				<div>{params.name}</div>
			</div>

			<div className={styles.input}>
				<Switch
					checked={checked}
					onChange={(event) => {
						if (params.onChange !== undefined) {
							params.onChange(!checked);
						}
						setChecked(event.currentTarget.checked);
					}}
				/>
			</div>
		</div>
	);
}
