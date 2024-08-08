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

export function usePipelinePanel(params: {
	setSelectedPipeline: Dispatch<SetStateAction<string | null>>;
	setSelectedDataset: Dispatch<SetStateAction<string | null>>;
	selectedDataset: string | null;
}) {
	const update_datasets = async (_: null) => {
		const res = await fetch("/api/dataset/list");
		const data: { name: string; ds_type: string }[] = await res.json();

		return data.map(({ name }) => {
			return {
				label: name,
				value: name,
				disabled: false,
			};
		});
	};

	const update_pipelines = async (dataset: string | null) => {
		if (dataset === null) {
			return Promise.resolve(null);
		}

		const res = await fetch(
			"/api/pipeline/list?" +
				new URLSearchParams({
					dataset,
				}),
		);

		const data: { input_type: string; name: string }[] = await res.json();

		return data.map(({ name, input_type }) => {
			return {
				label: name,
				value: name,
				disabled: input_type == "None",
			};
		});
	};

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
