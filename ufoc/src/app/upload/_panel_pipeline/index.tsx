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
import { ApiSelector } from "@/app/components/apiselect";
import { update_datasets, update_pipelines } from "@/app/_util/select";

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
					<ApiSelector
						onSelect={params.setSelectedDataset}
						update_params={null}
						update_list={update_datasets}
						messages={{
							nothingmsg_normal: "No datasets found",
							nothingmsg_empty: "No datasets are available",
							placeholder_error: "could not fetch datasets",
							placeholder_normal: "select dataset",
							message_loading: "fetching datasets...",
						}}
					/>

					<ApiSelector
						onSelect={params.setSelectedPipeline}
						update_params={params.selectedDataset}
						update_list={update_pipelines}
						messages={{
							nothingmsg_normal: "No pipelines found",
							nothingmsg_empty: "This dataset has no pipelines",
							placeholder_error: "could not fetch pipelines",
							placeholder_normal: "select pipeline",
							message_null: "select a pipeline",
							message_loading: "fetching pipelines...",
						}}
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
