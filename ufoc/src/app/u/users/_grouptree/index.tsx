import styles from "./grouptree.module.scss";
import { Panel } from "@/app/components/panel";

import {
	XIconDots,
	XIconEdit,
	XIconGroup,
	XIconLock,
	XIconPlus,
	XIconTrash,
} from "@/app/components/icons";
import { useTree, TreeNode } from "@/app/components/tree";
import { useCallback, useEffect, useState } from "react";
import { ActionIcon, Menu, rem } from "@mantine/core";

type TreeState = {
	error: boolean;
	loading: boolean;
};

export type GroupId = { type: "RootGroup" } | { type: "Group"; id: number };

export type UserInfo = {
	id: number;
	name: string;
	group: GroupInfo;
};

export type GroupInfo = {
	name: string;
	id: GroupId;
	parent: GroupId | null;
};

export type GroupData = {
	group_info: GroupInfo;
	users: UserInfo[];
};

export function useGroupTreePanel() {
	const [treeState, setTreeState] = useState<TreeState>({
		error: false,
		loading: true,
	});

	const {
		node: GroupTree,
		data: treeData,
		setTreeData,
		selected,
	} = useTree<GroupData>({ defaultOpen: true });

	const update_tree = useCallback(() => {
		setTreeState((td) => {
			return {
				error: false,
				loading: true,
			};
		});

		fetch("/api/auth/group/list")
			.then((res) => res.json())
			.then((data: GroupData[]) => {
				const out: TreeNode<GroupData>[] = [];
				for (let i = 0; i < data.length; i++) {
					const g = data[i];
					const id =
						g.group_info.id.type === "RootGroup"
							? "RootGroup"
							: `${g.group_info.id.id}`;
					const parent =
						g.group_info.parent === null
							? "RootGroup"
							: g.group_info.parent.type === "RootGroup"
							? "RootGroup"
							: `${g.group_info.parent.id}`;

					let parent_idx = out.findIndex((x) => x.uid === parent);

					out.push({
						icon: <XIconGroup />,
						text: g.group_info.name,
						right: <GroupMenu group_id={g.group_info.id} />,
						selectable: true,
						uid: id,
						parent: parent_idx === -1 ? null : parent_idx,
						can_have_children: true,
						data: g,
					});
				}

				console.log(out);

				setTreeData(out);
			});
	}, [setTreeData]);

	useEffect(() => {
		update_tree();
	}, [update_tree]);

	const node = (
		<Panel
			panel_id={styles.panel_grouptree}
			icon={<XIconGroup />}
			title={"Manage Groups"}
		>
			<div className={styles.grouptree_container}>{GroupTree}</div>
		</Panel>
	);

	// TODO: return tree state
	return { node, selected, treeData };
}

function GroupMenu(params: { group_id: GroupId }) {
	return (
		<>
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Edit group</Menu.Label>
					<Menu.Item
						disabled={params.group_id.type === "RootGroup"}
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
					>
						Add subgroup
					</Menu.Item>

					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						disabled={params.group_id.type === "RootGroup"}
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Delete this group
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
