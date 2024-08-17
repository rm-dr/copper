"use client";

import styles from "./page.module.scss";
import { useInfoPanel } from "./_panel_info";
import { useUiPanel } from "./_panel_ui";

export default function Page() {
	const { node: infoPanel } = useInfoPanel({});
	const { node: uiPanel } = useUiPanel({});

	return (
		<main className={styles.main}>
			{infoPanel}
			{uiPanel}
		</main>
	);
}
