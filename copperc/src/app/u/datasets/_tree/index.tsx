import styles from "./tree.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import { ActionIcon, Button, Loader, Menu, Text, rem } from "@mantine/core";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { useNewDsModal } from "./modals/adddataset";
import { useTree, TreeNode } from "@/app/components/tree";
import { datasetTypes } from "@/app/_util/datasets";
import { attrTypes } from "@/app/_util/attrs";
import { useDeleteAttrModal } from "./modals/delattr";
import { useAddAttrModal } from "./modals/addattr";
import { useDeleteClassModal } from "./modals/delclass";
import { useAddClassModal } from "./modals/addclass";
import { useDeleteDatasetModal } from "./modals/delds";
import {
	IconDatabase,
	IconDatabasePlus,
	IconDatabaseX,
	IconDots,
	IconEdit,
	IconFolder,
	IconFolderPlus,
	IconPlus,
	IconSettings,
	IconTrash,
	IconX,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";
import { useRenameAttrModal } from "./modals/renameattr";
import { useRenameClassModal } from "./modals/renameclass";
import { useRenameDatasetModal } from "./modals/renamedataset";

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

export function TreePanel(params: {}) {
	const [treeState, setTreeState] = useState<TreeState>({
		error: false,
		loading: true,
	});

	const { node: DatasetTree, data: treeData, setTreeData } = useTree({});

	// TODO: move to function
	const update_tree = useCallback(() => {
		setTreeState((td) => {
			return {
				error: false,
				loading: true,
			};
		});

		APIclient.GET("/dataset/list")
			.then(({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}

				return Promise.all(
					data.map(async ({ ds_type, name: dataset }) => {
						const { data, error } = await APIclient.GET("/class/list", {
							params: {
								query: {
									dataset,
								},
							},
						});

						if (error !== undefined) {
							throw error;
						}

						return {
							name: dataset,
							type: ds_type,
							classes: data,
						};
					}),
				);
			})
			.then((data) => {
				const tree_data: TreeNode<null>[] = [];
				for (const d of data) {
					let d_type = datasetTypes.find((x) => x.serialize_as === d.type);
					const d_node = tree_data.push({
						icon: d_type?.icon,
						text: d.name,
						right: (
							<DatasetMenu dataset_name={d.name} onSuccess={update_tree} />
						),
						icon_tooltip: {
							content: d_type?.pretty_name,
							position: "left",
						},
						selectable: false,
						uid: `dataset-${d.name}`,
						parent: null,
						can_have_children: true,
						data: null,
					});

					for (const c of d.classes) {
						const c_node = tree_data.push({
							icon: <XIcon icon={IconFolder} />,
							text: c.name,
							right: (
								<ClassMenu
									dataset_name={d.name}
									class={c}
									onSuccess={update_tree}
								/>
							),
							selectable: false,
							uid: `dataset-${d.name}-class-${c.name}`,
							parent: d_node - 1,
							can_have_children: true,
							data: null,
						});

						for (const a of c.attrs) {
							let a_type = attrTypes.find(
								(x) => x.serialize_as === a.data_type.type,
							);
							tree_data.push({
								icon: a_type?.icon,
								text: a.name,
								right: (
									<AttrMenu
										dataset_name={d.name}
										class={c}
										attr={a}
										onSuccess={update_tree}
									/>
								),
								icon_tooltip: {
									content: a_type?.pretty_name,
									position: "left",
								},
								selectable: false,
								uid: `dataset-${d.name}-class-${c.name}-attr-${a.name}`,
								parent: c_node - 1,
								can_have_children: false,
								data: null,
							});
						}
					}
				}

				setTreeData(tree_data);
				setTreeState({
					error: false,
					loading: false,
				});
			})
			.catch(() => {
				setTreeState({
					error: true,
					loading: false,
				});
			});
	}, [setTreeData]);

	const { open: openModal, modal: newDsModal } = useNewDsModal(update_tree);

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
					Could not fetch datasets
				</Text>
			</Wrapper>
		);
	} else if (treeData.length === 0) {
		tree = (
			<Wrapper>
				<XIcon
					icon={IconDatabaseX}
					style={{
						height: "5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				/>
				<Text size="lg" c="dimmed">
					No datasets
				</Text>
			</Wrapper>
		);
	} else {
		tree = DatasetTree;
	}

	return (
		<>
			{newDsModal}
			<Panel
				panel_id={styles.panel_tree as string}
				icon={<XIcon icon={IconDatabaseX} />}
				title={"Manage datasets"}
			>
				<PanelTitle
					icon={<XIcon icon={IconSettings} />}
					title={"Control Panel"}
				/>
				<Button
					radius="0"
					onClick={() => {
						openModal();
					}}
					variant="light"
					color="green"
					fullWidth
					leftSection={<XIcon icon={IconDatabasePlus} />}
					style={{ cursor: "default" }}
				>
					Create a new dataset
				</Button>

				<PanelTitle icon={<XIcon icon={IconDatabase} />} title={"Datasets"} />
				<div className={styles.dataset_list}>{tree}</div>
			</Panel>
		</>
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

	const { open: openRename, modal: modalRename } = useRenameDatasetModal({
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddClass}
			{modalRename}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIcon icon={IconDots} style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Dataset</Menu.Label>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconEdit}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openRename}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconFolderPlus}
								style={{ width: rem(14), height: rem(14) }}
							/>
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
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
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

function ClassMenu(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteClassModal({
		dataset_name: params.dataset_name,
		class: params.class,
		onSuccess: params.onSuccess,
	});

	const { open: openAddAttr, modal: modalAddAttr } = useAddAttrModal({
		dataset_name: params.dataset_name,
		class: params.class,
		onSuccess: params.onSuccess,
	});

	const { open: openRename, modal: modalRename } = useRenameClassModal({
		dataset_name: params.dataset_name,
		class: params.class,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalRename}
			{modalAddAttr}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIcon icon={IconDots} style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Class</Menu.Label>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconEdit}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openRename}
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
						onClick={openAddAttr}
					>
						Add attribute
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
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

function AttrMenu(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	attr: components["schemas"]["AttrInfo"];
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteAttrModal({
		dataset_name: params.dataset_name,
		class: params.class,
		attr: params.attr,
		onSuccess: params.onSuccess,
	});

	const { open: openRename, modal: modalRename } = useRenameAttrModal({
		dataset_name: params.dataset_name,
		attr: params.attr,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalRename}
			{modalDelete}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIcon icon={IconDots} style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Attribute</Menu.Label>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconEdit}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openRename}
					>
						Rename
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openDelete}
					>
						Delete this attribute
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
