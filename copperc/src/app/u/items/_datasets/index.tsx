import styles from "./datasets.module.scss";
import { Panel } from "@/components/panel";

import { DatasetSelector } from "@/components/apiselect/dataset";
import { ClassSelector } from "@/components/apiselect/class";
import { XIcon } from "@/components/icons";
import { IconDatabase } from "@tabler/icons-react";

export function DatasetPanel(params: {
	dataset: string | null;
	class: (
		v:
			| {
					dataset: string;
					class_idx: number | null;
			  }
			| { dataset: null; class_idx: null },
	) => void;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_datasets}
				icon={<XIcon icon={IconDatabase} />}
				title={"Select dataset"}
			>
				<DatasetSelector
					onSelect={(v) => params.class({ dataset: v, class_idx: null })}
				/>
				<ClassSelector
					key={params.dataset}
					onSelect={(class_idx) => {
						params.class(
							params.dataset === null
								? { dataset: null, class_idx: null }
								: {
										dataset: params.dataset,
										class_idx,
								  },
						);
					}}
					selectedDataset={params.dataset}
				/>
			</Panel>
		</>
	);
}
