"use client";

import { TreePanel } from ".";
import styles from "./page.module.scss";

export default function Page() {
	return (
		<div className={styles.main}>
			<TreePanel />
		</div>
	);
}
