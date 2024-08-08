import styles from "./grouptree.module.scss";
import { Panel } from "@/app/components/panel";
import { useTree, TreeNode } from "@/app/components/tree";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { ActionIcon, Loader, Menu, Text, rem } from "@mantine/core";
import { useAddGroupModal } from "../_modals/addgroup";
import { useDeleteGroupModal } from "../_modals/delgroup";
import { XIcon } from "@/app/components/icons";
import {
	IconDots,
	IconEdit,
	IconPlus,
	IconTrash,
	IconUsersGroup,
	IconX,
} from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";
import { useRenameGroupModal } from "../_modals/renamegroup";

type TreeState = {
	error: boolean;
	loading: boolean;
};

const Wrapper = (params: { children: ReactNode }) => {
	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				width: "100%",
				marginTop: "2rem",
				marginBottom: "2rem",
				userSelect: "none",
			}}
		>
			<div
				style={{
					display: "block",
					textAlign: "center",
				}}
			>
				{params.children}
			</div>
		</div>
	);
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
	} = useTree<components["schemas"]["ListgroupInfo"]>({ defaultOpen: true });

	const update_tree = useCallback(() => {
		setTreeState((td) => {
			return {
				error: false,
				loading: true,
			};
		});

		APIclient.GET("/auth/group/list")
			.then(({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}

				const out: TreeNode<components["schemas"]["ListgroupInfo"]>[] = [];
				for (let i = 0; i < data.length; i++) {
					const g = data[i];

					// Not possible
					if (g === undefined) {
						continue;
					}

					const id =
						g.group_info.id.type === "RootGroup"
							? "RootGroup"
							: `${g.group_info.id.id}`;

					const parent =
						g.group_info.parent === undefined || g.group_info.parent === null
							? "RootGroup"
							: // apiclient has odd behavior, these `as` are a hack
							(
									g.group_info.parent as {
										type: string;
									}
							  ).type === "RootGroup"
							? "RootGroup"
							: (
									g.group_info.parent as {
										id: number;
										type: string;
									}
							  ).id.toString();

					let parent_idx = out.findIndex((x) => x.uid === parent);

					out.push({
						icon: <XIcon icon={IconUsersGroup} />,
						text: g.group_info.name,
						right: <GroupMenu group={g.group_info} onChange={update_tree} />,
						selectable: true,
						uid: id,
						parent: parent_idx === -1 ? null : parent_idx,
						can_have_children: true,
						data: g,
					});
				}

				setTreeData(out);

				setTreeState((td) => {
					return {
						error: false,
						loading: false,
					};
				});
			})
			.catch(() => {
				setTreeState((td) => {
					return {
						error: true,
						loading: false,
					};
				});
			});
	}, [setTreeData]);

	useEffect(() => {
		update_tree();
	}, [update_tree]);

	let tree;
	if (treeState.loading) {
		tree = (
			<Wrapper>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						height: "5rem",
					}}
				>
					<Loader color="dimmed" size="4rem" />
				</div>
				<Text size="lg" c="dimmed">
					Loading...
				</Text>
			</Wrapper>
		);
	} else if (treeState.error) {
		tree = (
			<Wrapper>
				<XIcon
					icon={IconX}
					style={{
						height: "5rem",
						color: "var(--mantine-color-red-7)",
					}}
				/>
				<Text size="lg" c="red">
					Could not fetch groups
				</Text>
			</Wrapper>
		);
	} else {
		tree = GroupTree;
	}

	const node = (
		<Panel
			panel_id={styles.panel_grouptree as string}
			icon={<XIcon icon={IconUsersGroup} />}
			title={"Manage Groups"}
		>
			<div className={styles.grouptree_container}>{tree}</div>
		</Panel>
	);

	// TODO: return tree state
	return { node, selected, treeData, reloadTree: update_tree };
}

function GroupMenu(params: {
	group: components["schemas"]["GroupInfo"];
	onChange: () => void;
}) {
	const { open: openAddGroupModal, modal: addGroupModal } = useAddGroupModal({
		group: params.group,
		onChange: params.onChange,
	});

	const { open: openDelGroupModal, modal: delGroupModal } = useDeleteGroupModal(
		{
			group: params.group,
			onChange: params.onChange,
		},
	);

	const { open: openRenameModal, modal: renameModal } = useRenameGroupModal({
		group: params.group,
		onChange: params.onChange,
	});

	return (
		<>
			{addGroupModal}
			{delGroupModal}
			{renameModal}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIcon icon={IconDots} style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Edit group</Menu.Label>
					<Menu.Item
						disabled={params.group.id.type === "RootGroup"}
						leftSection={
							<XIcon
								icon={IconEdit}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openRenameModal}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconPlus}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openAddGroupModal}
					>
						Add subgroup
					</Menu.Item>

					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						disabled={params.group.id.type === "RootGroup"}
						leftSection={
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openDelGroupModal}
					>
						Delete this group
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
