import { ReactNode } from "react";
import styles from "./slider.module.scss";
import clsx from "clsx";

export function Slider(params: {
	icon: ReactNode;
	icon_text: string;
	text: string;
	right: ReactNode;

	is_selected: boolean;
	is_clickable: boolean;

	onClick?: () => void;
}) {
	return (
		<div
			className={clsx(
				styles.slider,
				params.is_clickable && styles.clickable,
				params.is_selected && styles.selected,
			)}
		>
			<div
				className={styles.slider_left}
				onMouseDown={(e) => {
					if (e.button == 0 && params.is_clickable) {
						if (params.onClick !== undefined) {
							params.onClick();
						}
					}
				}}
			>
				<div className={styles.slider_left_icon}>{params.icon}</div>
				<div className={styles.slider_left_text}>{params.icon_text}</div>
			</div>
			<div
				className={styles.slider_text}
				onMouseDown={(e) => {
					if (e.button == 0 && params.is_clickable) {
						if (params.onClick !== undefined) {
							params.onClick();
						}
					}
				}}
			>
				{params.text}
			</div>
			<div className={styles.slider_right}>{params.right}</div>
		</div>
	);
}
