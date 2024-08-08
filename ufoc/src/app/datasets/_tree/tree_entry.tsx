import { ReactNode } from "react";
import styles from "./tree_entry.module.scss";
import clsx from "clsx";
import { XIconListArrow } from "@/app/components/icons";

export function TreeEntry(params: {
	icon: ReactNode;
	icon_text: string;
	text: string;
	right: ReactNode;

	is_selected: boolean;
	is_clickable: boolean;
	expanded?: boolean;
	left_width: string;

	onClick?: () => void;
}) {
	return (
		<div
			className={clsx(
				styles.tree_entry,
				params.is_clickable && styles.clickable,
				params.is_selected && styles.selected,
				params.expanded === true && styles.expanded,
			)}
		>
			<div
				className={styles.tree_entry_arrow}
				onMouseDown={(e) => {
					if (e.button == 0 && params.is_clickable) {
						if (params.onClick !== undefined) {
							params.onClick();
						}
					}
				}}
			>
				{params.expanded === undefined ? null : <XIconListArrow />}
			</div>

			<div
				className={styles.tree_entry_left}
				onMouseDown={(e) => {
					if (e.button == 0 && params.is_clickable) {
						if (params.onClick !== undefined) {
							params.onClick();
						}
					}
				}}
			>
				<div className={styles.tree_entry_left_icon}>{params.icon}</div>
				<div
					className={styles.tree_entry_left_text}
					style={{ width: params.left_width }}
				>
					{params.icon_text}
				</div>
			</div>
			<div
				className={styles.tree_entry_text}
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
			<div className={styles.tree_entry_right}>{params.right}</div>
		</div>
	);
}
