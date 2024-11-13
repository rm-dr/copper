"use client";

import styles from "./items.module.scss";
import { useState } from "react";

import { components } from "@/lib/api/openapi";
import { ControlPanel } from "./_panels/controlpanel";
import { ItemTablePanel } from "./_panels/itemtable";
import { EditPanel } from "./_panels/editpanel";

export default function Page() {
	const [selectedItems, setSelectedItems] = useState<
		components["schemas"]["ItemlistItemInfo"][]
	>([]);

	const [selectedDataset, setSelectedDataset] = useState<
		components["schemas"]["DatasetInfo"] | null
	>(null);

	const [selectedClass, setSelectedClass] = useState<
		components["schemas"]["ClassInfo"] | null
	>(null);

	return (
		<>
			<div className={styles.main_top}>
				<ControlPanel
					selectedClass={selectedClass}
					setSelectedClass={setSelectedClass}
					setSelectedDataset={setSelectedDataset}
				/>

				<ItemTablePanel
					// Key is important, make sure we fully re-generate the table
					// when we select a new class
					key={`${selectedClass?.id}`}
					class={selectedClass}
					dataset={selectedDataset}
					setSelectedItems={setSelectedItems}
				/>
			</div>

			<div className={styles.main_bottom}>
				<EditPanel class={selectedClass} selectedItems={selectedItems} />
			</div>
		</>
	);
}
