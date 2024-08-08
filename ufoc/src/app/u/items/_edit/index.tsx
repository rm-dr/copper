import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { ActionIcon, Button, Text } from "@mantine/core";
import { ppBytes } from "@/app/_util/ppbytes";
import { useEffect, useState } from "react";
import {
	IconArrowRight,
	IconBinary,
	IconEdit,
	IconTrash,
	IconUpload,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";

export function EditPanel(params: {
	data: ItemData;
	select: Selected;
	class_attrs: { [attr: string]: any };
}) {
	const selectedItem =
		params.select.selected[0] === undefined
			? undefined
			: params.data.data[params.select.selected[0]];

	const [panelAttr, setPanelAttr] = useState<{
		name: string;
		value: any;
	} | null>(null);

	useEffect(() => {
		let selected = null;
		for (const [attr_name, value] of Object.entries(
			params.class_attrs,
		).sort()) {
			const d = attrTypes.find((x) => {
				return x.serialize_as === value.type;
			});
			// When changing class / dataset, select the first
			// panel-display attribute (if there is one)
			if (d?.editor.type === "panel") {
				selected = {
					name: attr_name,
					value,
				};
				break;
			}
		}

		setPanelAttr(selected);
	}, [params.data.class, params.data.dataset, params.class_attrs]);

	const selected_attr_spec = attrTypes.find((x) => {
		return x.serialize_as === panelAttr?.value.type;
	});

	const panel_data =
		selectedItem === undefined ||
		selected_attr_spec === undefined ||
		panelAttr === null ||
		selected_attr_spec.editor.type !== "panel" ? null : (
			<>
				<div className={styles.paneltitle}>
					<div className={styles.paneltitle_icon}>
						<XIcon icon={IconBinary} />
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
							<XIcon icon={IconTrash} style={{ width: "70%", height: "70%" }} />
						</ActionIcon>
					</div>
					<div>
						<ActionIcon variant="filled">
							<XIcon
								icon={IconUpload}
								style={{ width: "70%", height: "70%" }}
							/>
						</ActionIcon>
					</div>
				</div>
			</>
		);

	return (
		<>
			<Panel
				panel_id={styles.panel_edititem as string}
				icon={<XIcon icon={IconEdit} />}
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
														variant={
															panelAttr?.name === attr ? "filled" : "outline"
														}
														fullWidth
														rightSection={<XIcon icon={IconArrowRight} />}
														onClick={() => {
															setPanelAttr({ name: attr, value: val });
														}}
													>
														{panelAttr?.name === attr
															? "Shown in panel"
															: "View in panel"}
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
