import styles from "../page.module.scss";
import { Dispatch, SetStateAction } from "react";
import { Panel, PanelTitle } from "@/app/components/panel";
import { PanelSwitch } from "@/app/components/panel/parts/switch";
import { PanelText } from "@/app/components/panel/parts/text";
import { DatasetSelector } from "@/app/components/apiselect/dataset";
import { PipelineSelector } from "@/app/components/apiselect/pipeline";
import { IconAdjustments, IconHexagon, IconSchema } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";

export function usePipelinePanel(params: {
	setSelectedPipeline: Dispatch<SetStateAction<string | null>>;
	setSelectedDataset: Dispatch<SetStateAction<string | null>>;
	selectedDataset: string | null;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_id_pipe}
				icon={<XIcon icon={IconSchema} />}
				title={"Pipeline"}
			>
				<PanelTitle
					icon={<XIcon icon={IconHexagon} />}
					title={"Select pipeline"}
				/>
				<DatasetSelector onSelect={params.setSelectedDataset} />
				<PipelineSelector
					onSelect={params.setSelectedPipeline}
					selectedDataset={params.selectedDataset}
				/>

				<PanelTitle
					icon={<XIcon icon={IconAdjustments} />}
					title={"Configure arguments"}
				/>
				<PanelSwitch
					name={"Save album art?"}
					onChange={() => console.log("TODO")}
				/>
				<PanelText
					name={"Genre"}
					placeholder={"Genre..."}
					onChange={() => console.log("TODO")}
				/>
			</Panel>
		</>
	);
}
