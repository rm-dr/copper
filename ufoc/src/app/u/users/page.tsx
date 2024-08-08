"use client";

import styles from "./page.module.scss";
import { useGroupTreePanel } from "./_grouptree";
import { UsersPanel } from "./_users";

export default function Page() {
	const {
		node: groupTree,
		selected: selectedGroup,
		treeData,
	} = useGroupTreePanel();

	return (
		<main className={styles.main}>
			{groupTree}
			<UsersPanel data={treeData} selected={selectedGroup} />
		</main>
	);
}
