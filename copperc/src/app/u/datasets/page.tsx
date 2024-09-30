"use client";

import { useTreePanel } from "./tree";
import styles from "./page.module.scss";
import { Plus } from "lucide-react";
import { Button } from "@mantine/core";
import { useAddDatasetModal } from "./_modals/adddataset";

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
					<div className={styles.tree_panel_top}>
						<div className={styles.tree_panel_title}>Manage datasets</div>
						<div className={styles.tree_panel_button}>
							<Button
								leftSection={<Plus />}
								variant="subtle"
								onClick={openAddDs}
							>
								Add dataset
							</Button>
						</div>
					</div>
					<div className={styles.tree_panel_content}>{tree}</div>
				</div>
			</div>
		</>
	);
}
