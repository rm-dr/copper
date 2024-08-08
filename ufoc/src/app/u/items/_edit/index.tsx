import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { ActionIcon, Button, Text } from "@mantine/core";
import { ppBytes } from "@/app/_util/ppbytes";
import { Fragment, useCallback, useMemo, useState } from "react";
import {
	IconArrowRight,
	IconBinary,
	IconEdit,
	IconTrash,
	IconUpload,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { components } from "@/app/_util/api/openapi";

export function useEditPanel(params: { data: ItemData; select: Selected }) {
	const selectedItems = params.data.data.filter((_, idx) =>
		params.select.selected.includes(idx),
	);

	// Select attributes that are the same across all selected items
	const { first_panel, attr_values } = useMemo(() => {
		let attr_values: {
			[attr: string]: components["schemas"]["ItemListData"] | null;
		} = {};

		for (const it of selectedItems) {
			for (const [attr, val] of Object.entries(it.attrs)) {
				// This should never happen.
				// Hack to satisfy ts
				if (val === undefined) {
					continue;
				}

				const ex = attr_values[attr];
				if (ex === undefined) {
					attr_values[attr] = val;
				} else if (ex !== null) {
					let existing =
						ex.type === "Reference"
							? ex.item
							: ex.type === "Blob"
							? ex.handle
							: ex.type === "Binary"
							? ex.size // TODO: use handle
							: ex.value;

					let check =
						val.type === "Reference"
							? val.item
							: val.type === "Blob"
							? val.handle
							: val.type === "Binary"
							? val.size // TODO: use handle
							: val.value;

					if (existing !== check) {
						attr_values[attr] = null;
					}
				}
			}
		}

		let first_panel = null;
		for (const [_, value] of Object.entries(attr_values).sort()) {
			const d = attrTypes.find((x) => {
				return x.serialize_as === (value as { type: string }).type;
			});
			// When changing class / dataset, select the first
			// panel-display attribute (if there is one)
			if (d?.editor.type === "panel") {
				first_panel = value;
				break;
			}
		}

		return { first_panel, attr_values };
	}, [selectedItems]);

	const [panelAttr, setPanelAttr] = useState<
		components["schemas"]["ItemListData"] | null
	>(null);

	// Called whenever we change class or dataset
	const on_change_list = useCallback(() => {
		console.log("changelist");
		setPanelAttr(null);
	}, []);

	const selected_attr_spec = attrTypes.find((x) => {
		return x.serialize_as === panelAttr?.attr.data_type.type;
	});

	const panel_data =
		selectedItems.length === 0 ||
		selected_attr_spec === undefined ||
		panelAttr === null ||
		params.data.class === null ||
		selected_attr_spec.editor.type !== "panel" ||
		attr_values[panelAttr.attr.name] === null ? null : (
			/* Key here is important, it makes sure we get a new panel each time we select an item */
			<Fragment
				key={`${params.data.dataset}-${params.data.class}-${selectedItems
					.map((x) => x.idx)
					.join(",")}`}
			>
				<div className={styles.paneltitle}>
					<div className={styles.paneltitle_icon}>
						<XIcon icon={IconBinary} />
					</div>
					<div className={styles.paneltitle_name}>{panelAttr.attr.name}</div>
				</div>
				<div className={styles.panelbody}>
					<a
						target="_blank"
						href={
							"/api/item/attr?" +
							new URLSearchParams({
								dataset: params.data.dataset || "",
								class: params.data.class.toString() || "",
								attr: panelAttr.attr.handle.toString(),
								item_idx: selectedItems[0].idx.toString(),
							})
						}
						rel="noopener noreferrer"
						style={{ width: "100%", height: "100%", cursor: "inherit" }}
					>
						<div className={styles.panelimage}>
							{selected_attr_spec.editor.panel_body({
								dataset: params.data.dataset || "",
								class: params.data.class.toString() || "",
								item_idx: selectedItems[0].idx,
								attr_value: panelAttr,
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
			</Fragment>
		);

	const node = (
		<>
			<Panel
				panel_id={styles.panel_edititem as string}
				icon={<XIcon icon={IconEdit} />}
				title={"Edit items"}
			>
				<div className={styles.edit_container_rows}>
					{selectedItems.length === 0
						? null
						: Object.entries(selectedItems[0].attrs)
								.sort()
								.map(([_, val]) => {
									if (val === undefined) {
										// Unreachable
										return null;
									}

									const d = attrTypes.find((x) => {
										return x.serialize_as === val.type;
									});
									if (d === undefined) {
										return null;
									}

									// If we haven't selected a panel, select the first panel-able attribute.
									//
									// first_panel is set earlier in this component. we can't use val, because
									// we'll then select the _last_ panel (every iter of `map` will call setPanelAttr).
									// This is a hack, but it's good enough for now.
									if (
										d.editor.type === "panel" &&
										panelAttr === null &&
										first_panel !== null
									) {
										setPanelAttr(first_panel);
									}

									if (attr_values[val.attr.handle.toString()] === null) {
										return (
											<div
												key={`${selectedItems.map((x, _) => x.idx).join(",")}-${
													val.attr.name
												}`}
												className={styles.editrow}
											>
												<div className={styles.editrow_icon}>{d.icon}</div>
												<div className={styles.editrow_name}>
													{val.attr.name}
												</div>
												<div className={styles.editrow_value_old}>
													<Text c="dimmed" fs="italic">
														differs
													</Text>
												</div>
												<div className={styles.editrow_value_new}>
													<Text c="dimmed" fs="italic">
														differs
													</Text>
												</div>
											</div>
										);
									}

									return (
										<div
											key={`${selectedItems.map((_, idx) => idx).join(",")}-${
												val.attr.name
											}`}
											className={styles.editrow}
										>
											<div className={styles.editrow_icon}>{d.icon}</div>
											<div className={styles.editrow_name}>{val.attr.name}</div>
											<div className={styles.editrow_value_old}>
												{d.editor.type === "inline" ? (
													d.editor.old_value({
														attr_value: val,
													})
												) : (
													<Button
														radius="0px"
														variant={
															panelAttr?.attr.handle === val.attr.handle
																? "filled"
																: "outline"
														}
														fullWidth
														rightSection={<XIcon icon={IconArrowRight} />}
														onClick={() => {
															setPanelAttr(val);
														}}
													>
														{panelAttr?.attr.handle === val.attr.handle
															? "Shown in panel"
															: "View in panel"}
													</Button>
												)}
											</div>
											<div className={styles.editrow_value_new}>
												{d.editor.type === "inline"
													? d.editor.new_value({
															attr_value: val,
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

	return {
		node,
		on_change_list,
	};
}
