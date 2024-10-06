"use client";

import { useTreePanel } from "./tree";
import styles from "./page.module.scss";
import { Plus } from "lucide-react";
import { Button } from "@mantine/core";
import { useAddDatasetModal } from "./_modals/adddataset";
import TitleBar from "@/components/title";

export default function Page() {
	const { tree, reload } = useTreePanel();

	const { open: openAddDs, modal: modalAddDs } = useAddDatasetModal({
		onSuccess: reload,
	});

	return (
		<>
			{modalAddDs}
			<div className={styles.main}>
				<div className={styles.tree_panel}>
					<TitleBar
						text="Manage datasets"
						right={
							<Button
								leftSection={<Plus />}
								variant="subtle"
								onClick={openAddDs}
							>
								Add dataset
							</Button>
						}
					/>
					<div className={styles.tree_panel_content}>{tree}</div>
				</div>
			</div>
		</>
	);
}
