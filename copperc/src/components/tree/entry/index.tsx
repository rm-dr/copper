import { ReactNode } from "react";
import styles from "./tree_entry.module.scss";
import clsx from "clsx";
import { ChevronDown } from "lucide-react";

export function TreeEntry(params: {
	left: ReactNode;
	body: ReactNode;
	right?: ReactNode;

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
				{!params.expandable ? null : <ChevronDown />}
			</div>

			<div className={styles.tree_entry_content}>
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
					<div className={styles.tree_entry_left}>{params.left}</div>
				</div>

				<div
					className={styles.tree_entry_body}
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
					{params.body}
				</div>
				<div className={styles.tree_entry_right}>{params.right}</div>
			</div>
		</div>
	);
}
