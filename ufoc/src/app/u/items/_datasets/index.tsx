import styles from "./datasets.module.scss";
import { Panel, PanelSection } from "@/app/components/panel";

import { XIconDatabase, XIconFolder } from "@/app/components/icons";
import { Dispatch, SetStateAction } from "react";
import { DatasetSelector } from "@/app/components/apiselect/dataset";
import { ClassSelector } from "@/app/components/apiselect/class";

export function DatsetPanel(params: {
	selectedDataset: string | null;
	setSelectedDataset: Dispatch<SetStateAction<string | null>>;
	setSelectedClass: Dispatch<SetStateAction<string | null>>;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_datasets}
				icon={<XIconDatabase />}
				title={"Select dataset"}
			>
				<PanelSection>
					<DatasetSelector onSelect={params.setSelectedDataset} />
					<ClassSelector
						onSelect={params.setSelectedClass}
						selectedDataset={params.selectedDataset}
					/>
				</PanelSection>
			</Panel>
		</>
	);
}