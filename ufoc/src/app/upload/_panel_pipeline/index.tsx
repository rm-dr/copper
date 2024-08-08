import { Select } from "@mantine/core";
import styles from "../page.module.scss";
import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Panel, PanelSection } from "../../components/panel";
import {
	XIconAdjustments,
	XIconCpu,
	XIconHex,
	XIconPipeline,
} from "@/app/components/icons";
import { PanelSwitch } from "@/app/components/panel/parts/switch";
import { PanelText } from "@/app/components/panel/parts/text";

export function usePipelinePanel(params: {
	setSelectedPipeline: Dispatch<SetStateAction<string | null>>;
}) {
	const PipelineSelector = usePipelineSelector(params.setSelectedPipeline);

	return (
		<>
			<Panel
				panel_id={styles.panel_id_pipe}
				icon={<XIconPipeline />}
				title={"Pipeline"}
			>
				<PanelSection icon={<XIconHex />} title={"Select pipeline"}>
					{PipelineSelector}
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

type PipelineSelectorData = {
	pipelines: {
		name: string;
		input_type: string;
	}[];
	error: boolean;
};

// Search for a pipeline
function usePipelineSelector(onPipelineSelect: (value: string | null) => void) {
	const [plSelectorState, setPlSelectorState] = useState<PipelineSelectorData>({
		pipelines: [],
		error: false,
	});

	// Periodically refresh pipeline list
	// (not strictly necessary, but this helps us recover from errors)
	useEffect(() => {
		const update_pipeline_list = () => {
			fetch("/api/pipelines")
				.then((res) => res.json())
				.then((data) => {
					setPlSelectorState({
						pipelines: data,
						error: false,
					});
				})
				.catch(() => {
					setPlSelectorState({
						pipelines: [],
						error: true,
					});
				});
		};

		// First call has no delay
		update_pipeline_list();
		const id = setInterval(update_pipeline_list, 10_000);
		return () => clearInterval(id);
	}, []);

	return (
		<Select
			nothingFoundMessage="No pipeline found..."
			placeholder={
				plSelectorState.error
					? "Error: could not fetch pipelines from server"
					: "select a pipeline..."
			}
			data={plSelectorState.pipelines.map(({ name, input_type }) => {
				return {
					value: name,
					label: name,
					disabled: input_type == "None",
				};
			})}
			onOptionSubmit={onPipelineSelect}
			onClear={() => {
				onPipelineSelect(null);
			}}
			comboboxProps={{
				transitionProps: { transition: "fade-down", duration: 200 },
			}}
			error={plSelectorState.error}
			disabled={plSelectorState.error}
			searchable
			clearable
		/>
	);
}
