import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import { Button, Text } from "@mantine/core";
import { Fragment, useCallback, useMemo, useState } from "react";
import { IconArrowRight, IconBinary, IconEdit } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { components } from "@/app/_util/api/openapi";

export function useEditPanel(params: { data: ItemData; select: Selected }) {
	const selectedItems = params.data.data.filter((_, idx) =>
		params.select.selected.includes(idx),
	);

	const {
		// The first attribute that must be shown in a panel, if any
		first_panel,

		// Attributes that are the same across all selected items
		attr_values,
	} = useMemo(() => {
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
			if (value === null) {
				continue;
			}

			const d = attrTypes.find((x) => {
				return x.serialize_as === value.type;
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

									let value_old;
									let value_new;
									if (attr_values[val.attr.handle.toString()] === null) {
										value_old = (
											<Text c="dimmed" fs="italic">
												differs
											</Text>
										);

										value_new = (
											<Text c="dimmed" fs="italic">
												differs
											</Text>
										);
									} else {
										value_old =
											d.editor.type === "inline" ? (
												d.editor.old_value({
													key: selectedItems.map((x) => x.idx).join(","),
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
											);

										value_new =
											d.editor.type === "inline"
												? d.editor.new_value({
														key: selectedItems
															.map(({ idx }, _) => idx)
															.join(","),
														attr_value: val,
														onChange: console.log,
												  })
												: null;
									}

									return (
										<div
											key={`${selectedItems.map((x) => x.idx).join(",")}-${
												val.attr.name
											}`}
											className={styles.editrow}
										>
											<div className={styles.editrow_icon}>{d.icon}</div>
											<div className={styles.editrow_name}>{val.attr.name}</div>
											<div className={styles.editrow_value_old}>
												{value_old}
											</div>
											<div className={styles.editrow_value_new}>
												{value_new}
											</div>
										</div>
									);
								})}
				</div>
				<EditPanel
					data={params.data}
					selectedItems={selectedItems}
					panelAttr={panelAttr}
				/>
			</Panel>
		</>
	);

	return {
		node,
		on_change_list,
	};
}

function EditPanel(params: {
	data: ItemData;
	selectedItems: components["schemas"]["ItemListItem"][];
	panelAttr: components["schemas"]["ItemListData"] | null;
}) {
	const selected_attr_spec = attrTypes.find((x) => {
		return x.serialize_as === params.panelAttr?.attr.data_type.type;
	});

	if (
		selected_attr_spec?.editor.type === "inline" ||
		params.data.class === null ||
		params.panelAttr === null ||
		params.selectedItems.length === 0
	) {
		return null;
	}

	const selected_attr_value =
		params.panelAttr === null
			? undefined
			: params.selectedItems[0].attrs[params.panelAttr.attr.handle.toString()];

	const body =
		selected_attr_value === undefined ||
		selected_attr_spec === undefined ? null : (
			<div className={styles.panelimage}>
				{selected_attr_spec.editor.panel_body({
					dataset: params.data.dataset || "",
					class: params.data.class.toString() || "",
					item_idx: params.selectedItems[0].idx,
					attr_value: selected_attr_value,
				})}
			</div>
		);

	const bottom =
		selected_attr_value === undefined || selected_attr_spec === undefined
			? null
			: selected_attr_spec.editor.panel_bottom({
					dataset: params.data.dataset || "",
					class: params.data.class.toString() || "",
					item_idx: params.selectedItems[0].idx,
					attr_value: selected_attr_value,
			  });

	return (
		/* Key here is important, it makes sure we get a new panel each time we select an item */
		<Fragment
			key={`${params.data.dataset}-${params.data.class}-${params.selectedItems
				.map((x) => x.idx)
				.join(",")}`}
		>
			<div className={styles.edit_container_panel}>
				<div className={styles.paneltitle}>
					<div className={styles.paneltitle_icon}>
						{selected_attr_spec?.icon}
					</div>
					<div className={styles.paneltitle_name}>
						{params.panelAttr?.attr.name}
					</div>
				</div>
				<div className={styles.panelbody}>{body}</div>
				<div className={styles.panelbottom}>{bottom}</div>
			</div>
		</Fragment>
	);
}
