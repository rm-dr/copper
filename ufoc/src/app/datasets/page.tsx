"use client";

import styles from "./page.module.scss";
import { TreePanel } from "./_tree";

export default function Page() {
	return (
		<main className={styles.main}>
			<TreePanel />
		</main>
	);
}
