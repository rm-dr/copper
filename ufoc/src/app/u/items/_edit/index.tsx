import styles from "./edit.module.scss";
import { Panel } from "@/app/components/panel";
import { ItemData, Selected, selectedClass } from "../page";
import { attrTypeInfo, attrTypes } from "@/app/_util/attrs";
import { Button, Text } from "@mantine/core";
import { Fragment, useCallback, useMemo, useState } from "react";
import { IconArrowRight, IconEdit, IconEyeOff } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { components } from "@/app/_util/api/openapi";

export function EditPanel(params: {
	sel: selectedClass;
	select: Selected;
	data: ItemData;
}) {
	const selectedItems = useMemo(
		() =>
			params.data.data.filter((_, idx) => params.select.selected.includes(idx)),
		[params.select.selected, params.data.data],
	);

	// Attributes that are the same across all selected items
	const attr_values: {
		[attr: string]: components["schemas"]["ItemListData"] | null;
	} = useMemo(() => {
		let attr_values: {
			[attr: string]: components["schemas"]["ItemListData"] | null;
		} = {};

		for (const it of selectedItems) {
			for (const [attr, val] of Object.entries(it.attrs)) {
				// This should never happen.
				// Hack to satisfy TS.
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
							? 1 // If two items are selected, binaries are always different.
							: // 1 and 2 are arbitrary different values.
							  ex.value;

					let check =
						val.type === "Reference"
							? val.item
							: val.type === "Blob"
							? val.handle
							: val.type === "Binary"
							? 2 // If two items are selected, binaries are always different.
							: val.value;

					if (existing !== check) {
						attr_values[attr] = null;
					}
				}
			}
		}

		return attr_values;
	}, [selectedItems]);

	const [panelAttr, setPanelAttr] = useState<
		components["schemas"]["AttrInfo"] | null
	>(() => {
		if (params.sel.attrs === null) {
			return null;
		}

		for (const a of params.sel.attrs.sort((av, bv) => av.idx - bv.idx)) {
			const d = attrTypes.find((x) => {
				return x.serialize_as === a.data_type.type;
			});

			// When changing class / dataset, select the first
			// panel-display attribute (if there is one)
			if (d?.editor.type === "panel") {
				return a;
			}
		}
		return null;
	});

	const title =
		selectedItems.length <= 1
			? "Edit items"
			: `Edit items (${selectedItems.length} selected)`;

	return (
		<>
			<Panel
				panel_id={styles.panel_edititem as string}
				icon={<XIcon icon={IconEdit} />}
				title={title}
			>
				<div className={styles.edit_container_rows}>
					{selectedItems.length === 0
						? null
						: Object.entries(selectedItems[0].attrs)
								.sort(
									([aa, av], [ba, bv]) =>
										(av as unknown as components["schemas"]["ItemListData"])
											.attr.idx -
										(bv as unknown as components["schemas"]["ItemListData"])
											.attr.idx,
								)
								.map(([_, val]) => {
									if (val === undefined) {
										return null; // Unreachable
									}

									let v = attr_values[val.attr.handle.toString()];
									return (
										<EditRow
											key={`${val.attr.handle}-
											${selectedItems.map((x) => x.idx).join(",")}`}
											item={selectedItems[0]}
											attr={val.attr}
											value_new={v}
											value_old={v}
											setPanelAttr={setPanelAttr}
											panelAttr={panelAttr}
										/>
									);
								})}
				</div>
				<EditSubPanel
					dataset={params.sel.dataset}
					class={params.sel.class_idx}
					selectedItems={selectedItems}
					attrValues={attr_values}
					panelAttr={panelAttr}
				/>
			</Panel>
		</>
	);
}

function EditRow(params: {
	attr: components["schemas"]["AttrInfo"];
	item: components["schemas"]["ItemListItem"];
	value_old: components["schemas"]["ItemListData"] | null;
	value_new: components["schemas"]["ItemListData"] | null;
	panelAttr: components["schemas"]["AttrInfo"] | null;
	setPanelAttr: (attr: components["schemas"]["AttrInfo"] | null) => void;
}) {
	const attr_spec = attrTypes.find((x) => {
		return x.serialize_as === params.attr.data_type.type;
	}) as attrTypeInfo;

	let value_old_component =
		params.value_old === null ? (
			<Text c="dimmed" fs="italic">
				differs
			</Text>
		) : attr_spec.editor.type === "inline" ? (
			attr_spec.editor.old_value({
				attr_value: params.value_old,
			})
		) : (
			<Button
				radius="0px"
				variant={
					params.panelAttr?.handle === params.attr.handle ? "filled" : "outline"
				}
				fullWidth
				rightSection={<XIcon icon={IconArrowRight} />}
				onClick={() => {
					params.setPanelAttr(params.attr);
				}}
			>
				{params.panelAttr?.handle === params.attr.handle
					? "Shown in panel"
					: "View in panel"}
			</Button>
		);

	let value_new_component =
		params.value_new === null ? (
			<Text c="dimmed" fs="italic">
				differs
			</Text>
		) : attr_spec.editor.type === "inline" ? (
			attr_spec.editor.new_value({
				attr_value: params.value_new,
				onChange: console.log,
			})
		) : null;

	return (
		<div className={styles.editrow}>
			<div className={styles.editrow_icon}>{attr_spec.icon}</div>
			<div className={styles.editrow_name}>{params.attr.name}</div>
			<div className={styles.editrow_value_old}>{value_old_component}</div>
			<div className={styles.editrow_value_new}>{value_new_component}</div>
		</div>
	);
}

function EditSubPanel(params: {
	dataset: string | null;
	class: number | null;
	selectedItems: components["schemas"]["ItemListItem"][];
	attrValues: { [attr: string]: components["schemas"]["ItemListData"] | null };
	panelAttr: components["schemas"]["AttrInfo"] | null;
}) {
	const selected_attr_spec = attrTypes.find((x) => {
		return x.serialize_as === params.panelAttr?.data_type.type;
	});

	if (
		selected_attr_spec?.editor.type === "inline" ||
		params.class === null ||
		params.panelAttr === null ||
		params.class === null ||
		params.dataset === null ||
		params.selectedItems.length === 0
	) {
		return null;
	}

	const selected_attr_value =
		params.panelAttr === null
			? null
			: params.attrValues[params.panelAttr.handle.toString()];

	const body =
		selected_attr_value === null ? (
			<div className={styles.panelbody_inner}>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						justifyContent: "center",
						alignItems: "center",
						height: "100%",
					}}
				>
					<div style={{ textAlign: "center" }}>
						<Text fs="italic" c="dimmed">
							<XIcon icon={IconEyeOff} style={{ height: "5rem" }} />
							Value differs among {params.selectedItems.length} selected items
						</Text>
					</div>
				</div>
			</div>
		) : selected_attr_spec === undefined ? null : (
			<div className={styles.panelbody_inner}>
				{selected_attr_spec.editor.panel_body({
					dataset: params.dataset,
					class: params.class,
					item_idx: params.selectedItems[0].idx,
					attr_value: selected_attr_value,
				})}
			</div>
		);

	const bottom =
		selected_attr_value === null || selected_attr_spec === undefined
			? null
			: selected_attr_spec.editor.panel_bottom({
					dataset: params.dataset,
					class: params.class,
					item_idx: params.selectedItems[0].idx,
					attr_value: selected_attr_value,
			  });

	return (
		/* Key here is important, it makes sure we get a new panel each time we select an item */
		<Fragment
			key={`${params.dataset}-${params.class}-${params.selectedItems
				.map((x) => x.idx)
				.join(",")}`}
		>
			<div className={styles.edit_container_panel}>
				<div className={styles.paneltitle}>
					<div className={styles.paneltitle_icon}>
						{selected_attr_spec?.icon}
					</div>
					<div className={styles.paneltitle_name}>{params.panelAttr?.name}</div>
				</div>
				<div className={styles.panelbody}>{body}</div>
				<div className={styles.panelbottom}>{bottom}</div>
			</div>
		</Fragment>
	);
}
