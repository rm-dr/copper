"use client";

import styles from "./page.module.scss";
import { DatsetPanel } from "./_datasets";
import { useState } from "react";
import { ItemTablePanel } from "./_itemtable";

export default function Page() {
	const [selectedDataset, setSelectedDataset] = useState<string | null>(null);

	return (
		<main className={styles.main}>
			<ItemTablePanel
				selectedDataset={selectedDataset}
				setSelectedDataset={setSelectedDataset}
			/>
			<DatsetPanel
				selectedDataset={selectedDataset}
				setSelectedDataset={setSelectedDataset}
			/>
		</main>
	);
}
