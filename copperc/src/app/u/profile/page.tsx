"use client";

import styles from "./page.module.scss";
import { useInfoPanel } from "./_panel_info";

export default function Page() {
	const { node: infoPanel } = useInfoPanel({});

	return <main className={styles.main}>{infoPanel}</main>;
}
