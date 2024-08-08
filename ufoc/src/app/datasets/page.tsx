"use client";

import styles from "./page.module.scss";
import { useDbList } from "./_dblist";
import { useEdit } from "./_edit";
import { useState } from "react";

export default function Page() {
	const [selectedDs, setSelectedDs] = useState<null | string>(null);

	const panel_dblist = useDbList(setSelectedDs, selectedDs);
	const panel_edit = useEdit(selectedDs);

	return (
		<main className={styles.main}>
			{panel_dblist}
			{panel_edit}
		</main>
	);
}
