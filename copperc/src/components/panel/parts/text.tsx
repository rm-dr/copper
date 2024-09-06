import { TextInput } from "@mantine/core";
import styles from "./parts.module.scss";

export function PanelText(params: {
	name?: string;
	placeholder?: string;
	onChange?: (value: string) => void;
}) {
	return (
		<div className={styles.panelpart}>
			<div className={styles.label}>
				<div>Genre</div>
			</div>

			<div className={styles.input}>
				<TextInput
					placeholder={
						params.placeholder === undefined ? "" : params.placeholder
					}
					size="xs"
					onChange={(event) => {
						if (params.onChange !== undefined) {
							params.onChange(event.currentTarget.value);
						}
					}}
				/>
			</div>
		</div>
	);
}
