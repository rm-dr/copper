import styles from "./datasets.module.scss";
import { Panel } from "@/app/components/panel";

import { XIconDatabase } from "@/app/components/icons";
import { DatasetSelector } from "@/app/components/apiselect/dataset";
import { ClassSelector } from "@/app/components/apiselect/class";

export function DatsetPanel(params: {
	selectedDataset: string | null;
	setSelectedDataset: (dataset: string | null) => void;
	setSelectedClass: (class_name: string | null) => void;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_datasets}
				icon={<XIconDatabase />}
				title={"Select dataset"}
			>
				<DatasetSelector onSelect={params.setSelectedDataset} />
				<ClassSelector
					onSelect={params.setSelectedClass}
					selectedDataset={params.selectedDataset}
				/>
			</Panel>
		</>
	);
}
