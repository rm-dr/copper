import styles from "./datasets.module.scss";
import { Panel } from "@/app/components/panel";

import { DatasetSelector } from "@/app/components/apiselect/dataset";
import { ClassSelector } from "@/app/components/apiselect/class";
import { XIcon } from "@/app/components/icons";
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
