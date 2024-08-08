import { XIconDots, XIconEdit, XIconTrash } from "@/app/components/icons";
import { ActionIcon, Menu, rem } from "@mantine/core";
import { TreeEntry } from "../tree_entry";
import { attrTypes } from "@/app/_util/attrs";
import { useDeleteAttrModal } from "./modals/delattr";

export function AttrList(params: {
	update_tree: () => void;
	dataset: string;
	class: string;
	open: boolean;

	attrs: {
		name: string;
		type: string;
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
			{params.attrs.map(({ name: attr_name, type: attr_type }) => {
				// Find attr icon
				let type_def = attrTypes.find((x) => {
					return x.serialize_as === attr_type;
				});

				return (
					<TreeEntry
						key={`dataset-${params.dataset}-class-${params.class}-attr-${attr_name}`}
						is_clickable={true}
						is_selected={false}
						onClick={() => {}}
						icon={type_def?.icon}
						text={attr_name}
						icon_tooltip={type_def?.pretty_name}
						icon_tooltip_position={"left"}
						right={
							<AttrMenu
								dataset_name={params.dataset}
								class_name={params.class}
								attr_name={attr_name}
								onSuccess={params.update_tree}
								disabled={!params.open}
							/>
						}
					/>
				);
			})}
		</div>
	);
}

function AttrMenu(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
	disabled: boolean;
	onSuccess: () => void;
}) {
	const { open: openDelAttr, modal: modalDelAttr } = useDeleteAttrModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		attr_name: params.attr_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelAttr}
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
					<Menu.Label>Attribute</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openDelAttr}
					>
						Delete this attribute
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
