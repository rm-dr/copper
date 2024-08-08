import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";

import {
	XIconArrowRight,
	XIconAttrBinary,
	XIconEdit,
	XIconTrash,
	XIconUpload,
} from "@/app/components/icons";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { ActionIcon, Button, Text } from "@mantine/core";
import { ppBytes } from "@/app/_util/ppbytes";
import { useEffect, useState } from "react";

export function EditPanel(params: { data: ItemData; select: Selected }) {
	const selectedItem = params.data.data[params.select.selected[0]];
	const [panelAttr, setPanelAttr] = useState<{
		name: string;
		value: any;
	} | null>(null);

	useEffect(() => {
		setPanelAttr(null);
	}, [params.data.class, params.data.dataset]);

	const selected_attr_spec = attrTypes.find((x) => {
		return x.serialize_as === panelAttr?.value.type;
	});

	const panel_data =
		selectedItem === null ||
		selectedItem === undefined ||
		selected_attr_spec === undefined ||
		panelAttr === null ||
		selected_attr_spec.editor.type !== "panel" ? null : (
			<>
				<div className={styles.paneltitle}>
					<div className={styles.paneltitle_icon}>
						<XIconAttrBinary />
					</div>
					<div className={styles.paneltitle_name}>{panelAttr.name}</div>
				</div>
				<div className={styles.panelbody}>
					<a
						target="_blank"
						href={
							"/api/item/attr?" +
							new URLSearchParams({
								dataset: params.data.dataset || "",
								class: params.data.class || "",
								attr: panelAttr.name || "",
								item_idx: selectedItem.idx.toString(),
							})
						}
						rel="noopener noreferrer"
						style={{ width: "100%", height: "100%", cursor: "inherit" }}
					>
						{/* Key here is important, it makes sure we get a new panel each time we select an item */}
						<div
							className={styles.panelimage}
							key={`${params.data.dataset}-${params.data.class}-${selectedItem.idx}`}
						>
							{selected_attr_spec.editor.panel_body({
								dataset: params.data.dataset || "",
								class: params.data.class || "",
								item_idx: selectedItem.idx,
								attr_name: panelAttr.name,
								attr_val: panelAttr.value,
							})}
						</div>
					</a>
				</div>
				<div className={styles.panelbottom}>
					<div>
						<Text>{ppBytes(100088)}</Text>
					</div>
					<div style={{ flexGrow: 1 }}>
						<Text ff="monospace">image/png</Text>
					</div>
					<div>
						<ActionIcon variant="filled" color="red">
							<XIconTrash style={{ width: "70%", height: "70%" }} />
						</ActionIcon>
					</div>
					<div>
						<ActionIcon variant="filled">
							<XIconUpload style={{ width: "70%", height: "70%" }} />
						</ActionIcon>
					</div>
				</div>
			</>
		);

	return (
		<>
			<Panel
				panel_id={styles.panel_edititem}
				icon={<XIconEdit />}
				title={"Edit items"}
			>
				<div className={styles.edit_container_rows}>
					{selectedItem === undefined
						? null
						: Object.entries(selectedItem.attrs)
								.sort()
								.map(([attr, val]: [string, any]) => {
									const d = attrTypes.find((x) => {
										return x.serialize_as === val.type;
									});
									if (d === undefined) {
										return null;
									}

									return (
										<div
											key={`${selectedItem.idx}-${attr}`}
											className={styles.editrow}
										>
											<div className={styles.editrow_icon}>{d.icon}</div>
											<div className={styles.editrow_name}>{attr}</div>
											<div className={styles.editrow_value_old}>
												{d.editor.type === "inline" ? (
													d.editor.old_value({ attr: val })
												) : (
													<Button
														radius="0px"
														variant="outline"
														fullWidth
														rightSection={<XIconArrowRight />}
														onClick={() => {
															setPanelAttr({ name: attr, value: val });
														}}
													>
														View in panel
													</Button>
												)}
											</div>
											<div className={styles.editrow_value_new}>
												{d.editor.type === "inline"
													? d.editor.new_value({
															attr: val,
															onChange: console.log,
													  })
													: null}
											</div>
										</div>
									);
								})}
				</div>
				<div className={styles.edit_container_panel}>{panel_data}</div>
			</Panel>
		</>
	);
}
