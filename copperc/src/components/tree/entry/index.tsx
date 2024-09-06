import { ReactNode } from "react";
import styles from "./tree_entry.module.scss";
import clsx from "clsx";
import { FloatingPosition, Tooltip } from "@mantine/core";
import { XIcon } from "../../icons";
import { IconChevronDown } from "@tabler/icons-react";

export function TreeEntry(params: {
	icon: ReactNode;
	icon_tooltip?: ReactNode;
	icon_tooltip_position?: FloatingPosition;
	text: string;
	right: ReactNode;

	is_selected: boolean;
	is_expanded: boolean;
	selectable: boolean;
	expandable: boolean;

	onExpandClick?: () => void;
	onSelectClick?: () => void;
}) {
	return (
		<div
			className={clsx(
				styles.tree_entry,
				params.is_selected && styles.selected,
				params.is_expanded && styles.expanded,
			)}
		>
			<div
				className={styles.tree_entry_arrow}
				onMouseDown={(e) => {
					// Arrow click always toggles expanded
					if (e.button == 0 && params.expandable) {
						if (params.onExpandClick !== undefined) {
							params.onExpandClick();
						}
					}
				}}
			>
				{!params.expandable ? null : <XIcon icon={IconChevronDown} />}
			</div>

			<div
				className={styles.tree_entry_left}
				// Icon click always toggles expanded
				onMouseDown={(e) => {
					if (e.button == 0 && params.expandable) {
						if (params.onExpandClick !== undefined) {
							params.onExpandClick();
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
				// Body click can toggle expanded or select
				onMouseDown={(e) => {
					if (e.button == 0) {
						if (params.selectable) {
							if (params.onSelectClick !== undefined) {
								params.onSelectClick();
							}
						} else if (params.expandable) {
							if (params.onExpandClick !== undefined) {
								params.onExpandClick();
							}
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
