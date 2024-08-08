import {
	XIconDots,
	XIconEdit,
	XIconFolder,
	XIconPlus,
	XIconTrash,
} from "@/app/components/icons";
import { ActionIcon, Menu, rem } from "@mantine/core";
import { Dispatch, SetStateAction } from "react";
import { AttrList } from "./attr";
import { TreeEntry } from "../tree_entry";
import { TreeData } from "..";
import { useAddAttrModal } from "./modals/addattr";
import { useDeleteClassModal } from "./modals/delclass";

export function ClassList(params: {
	update_tree: () => void;
	open: boolean;
	setTreeData: Dispatch<SetStateAction<TreeData>>;
	dataset_name: string;
	dataset_idx: number;
	classes: {
		name: string;
		open: boolean;
		attrs: {
			name: string;
			type: string;
		}[];
	}[];
}) {
	return (
		<div
			style={{
				paddingLeft: "2rem",
				transition: "200ms",
				display: params.open ? "" : "none",
			}}
		>
			{params.classes.map(
				({ name: class_name, open: class_open, attrs }, idx) => {
					return (
						<div key={`dataset-${params.dataset_name}-class-${class_name}`}>
							<TreeEntry
								is_clickable={true}
								is_selected={class_open}
								onClick={() => {
									params.setTreeData((x) => {
										let t = { ...x };
										if (t.datasets !== null) {
											t.datasets[params.dataset_idx].classes[idx].open =
												!class_open;
										}
										return t;
									});
								}}
								icon={<XIconFolder />}
								text={class_name}
								expanded={class_open}
								right={
									<ClassMenu
										dataset_name={params.dataset_name}
										class_name={class_name}
										onSuccess={params.update_tree}
										disabled={!params.open}
									/>
								}
							/>
							<AttrList
								update_tree={params.update_tree}
								open={class_open}
								dataset={params.dataset_name}
								class={class_name}
								attrs={attrs}
							/>
						</div>
					);
				},
			)}
		</div>
	);
}

function ClassMenu(params: {
	dataset_name: string;
	class_name: string;
	disabled: boolean;
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteClassModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddAttr, modal: modalAddAttr } = useAddAttrModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddAttr}
			<Menu
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
				disabled={params.disabled}
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Class</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconPlus style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openAddAttr}
					>
						Add attribute
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
						Delete this class
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
