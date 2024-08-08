import {
	XIconDots,
	XIconEdit,
	XIconFolderPlus,
	XIconTrash,
} from "@/app/components/icons";
import { ActionIcon, Button, Menu, Text, TextInput, rem } from "@mantine/core";

import { TreeData } from "..";
import { Dispatch, SetStateAction, useState } from "react";

import styles from "../tree.module.scss";
import { ClassList } from "./class";
import { TreeEntry } from "../tree_entry";
import { datasetTypes } from "../datasets";
import { useDeleteDatasetModal } from "./modals/delds";
import { useAddClassModal } from "./modals/addclass";

export function DatasetList(params: {
	update_tree: () => void;
	datasets: {
		name: string;
		type: string;
		open: boolean;
		classes: {
			name: string;
			open: boolean;
			attrs: {
				name: string;
				type: string;
			}[];
		}[];
	}[];
	setTreeData: Dispatch<SetStateAction<TreeData>>;
}) {
	return (
		<div className={styles.dataset_list}>
			{params.datasets.map(
				(
					{
						name: dataset_name,
						type: dataset_type,
						open: dataset_open,
						classes,
					},
					idx,
				) => {
					// Find dataset icon
					let ds_type_def = datasetTypes.find((x) => {
						return x.serialize_as === dataset_type;
					});

					return (
						<div
							key={`dataset-${dataset_name}`}
							style={{
								paddingLeft: "0",
							}}
						>
							<TreeEntry
								is_clickable={true}
								is_selected={dataset_open}
								onClick={() => {
									params.setTreeData((x) => {
										let t = { ...x };
										if (t.datasets !== null) {
											t.datasets[idx].open = !dataset_open;
										}
										return t;
									});
								}}
								icon={ds_type_def?.icon}
								text={dataset_name}
								icon_tooltip={ds_type_def?.pretty_name}
								icon_tooltip_position={"top"}
								expanded={dataset_open}
								right={
									<DatasetMenu
										dataset_name={dataset_name}
										onSuccess={params.update_tree}
									/>
								}
							/>
							<ClassList
								open={dataset_open}
								update_tree={params.update_tree}
								setTreeData={params.setTreeData}
								dataset_name={dataset_name}
								dataset_idx={idx}
								classes={classes}
							/>
						</div>
					);
				},
			)}
		</div>
	);
}

function DatasetMenu(params: { dataset_name: string; onSuccess: () => void }) {
	const { open: openDelete, modal: modalDelete } = useDeleteDatasetModal({
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddClass, modal: modalAddClass } = useAddClassModal({
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddClass}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Dataset</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconFolderPlus style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openAddClass}
					>
						Add class
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openDelete}
					>
						Delete this dataset
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
