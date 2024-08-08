"use client";

import styles from "./page.module.scss";
import { DatsetPanel } from "./_datasets";
import { useState } from "react";
import { ItemTablePanel } from "./_itemtable";

export default function Page() {
	const [selectedDataset, setSelectedDataset] = useState<string | null>(null);
	const [selectedClass, setSelectedClass] = useState<string | null>(null);

	return (
		<main className={styles.main}>
			<div className={styles.wrap_top}>
				<div className={styles.wrap_list}>
					<ItemTablePanel
						selectedDataset={selectedDataset}
						selectedClass={selectedClass}
					/>
				</div>
				<div className={styles.wrap_right}>
					<DatsetPanel
						selectedDataset={selectedDataset}
						setSelectedDataset={setSelectedDataset}
						setSelectedClass={setSelectedClass}
					/>
				</div>
			</div>
			<div className={styles.wrap_bottom}>
				<DatsetPanel
					selectedDataset={selectedDataset}
					setSelectedDataset={setSelectedDataset}
					setSelectedClass={setSelectedClass}
				/>
			</div>
		</main>
	);
}
