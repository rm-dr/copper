import styles from "./datasets.module.scss";
import { Panel, PanelSection } from "../../components/panel";

import { XIconDatabase, XIconFolder } from "@/app/components/icons";
import { ApiSelector } from "@/app/components/apiselect";
import { update_classes, update_datasets } from "@/app/_util/select";
import { Dispatch, SetStateAction } from "react";

export function DatsetPanel(params: {
	selectedDataset: string | null;
	setSelectedDataset: Dispatch<SetStateAction<string | null>>;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_datasets}
				icon={<XIconDatabase />}
				title={"Select dataset"}
			>
				<PanelSection>
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
						onSelect={console.log}
						update_params={params.selectedDataset}
						update_list={update_classes}
						messages={{
							nothingmsg_normal: "No classes found",
							nothingmsg_empty: "This dataset has no classes",
							placeholder_error: "could not fetch classes",
							placeholder_normal: "select a class",
							message_null: "select a class",
							message_loading: "fetching classes...",
						}}
					/>
				</PanelSection>
			</Panel>
		</>
	);
}
