import { ReactNode } from "react";
import styles from "./tree_entry.module.scss";
import clsx from "clsx";
import { XIconListArrow } from "@/app/components/icons";
import { FloatingPosition, Text, Tooltip } from "@mantine/core";

export function TreeEntry(params: {
	icon: ReactNode;
	icon_tooltip?: ReactNode;
	icon_tooltip_position?: FloatingPosition;
	text: string;
	right: ReactNode;

	is_selected: boolean;
	is_clickable: boolean;
	expanded?: boolean;

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
				{params.icon_tooltip === undefined ? (
					<div className={styles.tree_entry_left_icon}>{params.icon}</div>
				) : (
					<Tooltip
						arrowOffset={0}
						arrowSize={8}
						withArrow
						position={params.icon_tooltip_position}
						color="gray"
						label={params.icon_tooltip}
						transitionProps={{ transition: "fade", duration: 200 }}
					>
						<div className={styles.tree_entry_left_icon}>{params.icon}</div>
					</Tooltip>
				)}
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
