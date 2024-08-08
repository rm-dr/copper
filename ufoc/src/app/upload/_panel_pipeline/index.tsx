import styles from "../page.module.scss";
import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Panel, PanelSection } from "../../components/panel";
import {
	XIconAdjustments,
	XIconHex,
	XIconPipeline,
} from "@/app/components/icons";
import { PanelSwitch } from "@/app/components/panel/parts/switch";
import { PanelText } from "@/app/components/panel/parts/text";
import { DatasetSelector } from "@/app/components/apiselect/dataset";
import { PipelineSelector } from "@/app/components/apiselect/pipeline";

export function usePipelinePanel(params: {
	setSelectedPipeline: Dispatch<SetStateAction<string | null>>;
	setSelectedDataset: Dispatch<SetStateAction<string | null>>;
	selectedDataset: string | null;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_id_pipe}
				icon={<XIconPipeline />}
				title={"Pipeline"}
			>
				<PanelSection icon={<XIconHex />} title={"Select pipeline"}>
					<DatasetSelector onSelect={params.setSelectedDataset} />
					<PipelineSelector
						onSelect={params.setSelectedPipeline}
						selectedDataset={params.selectedDataset}
					/>
				</PanelSection>

				<PanelSection icon={<XIconAdjustments />} title={"Configure arguments"}>
					<PanelSwitch name={"Save album art?"} onChange={console.log} />
					<PanelText
						name={"Genre"}
						placeholder={"Genre..."}
						onChange={console.log}
					/>
				</PanelSection>
			</Panel>
		</>
	);
}
