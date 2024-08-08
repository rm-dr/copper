import styles from "./tree.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import {
	XIconDatabase,
	XIconDatabasePlus,
	XIconDatabaseX,
	XIconDots,
	XIconEdit,
	XIconFolder,
	XIconFolderPlus,
	XIconPlus,
	XIconSettings,
	XIconTrash,
	XIconX,
} from "@/app/components/icons";
import { ActionIcon, Button, Loader, Menu, Text, rem } from "@mantine/core";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { useNewDsModal } from "./modals/addds";
import { useTree, TreeNode } from "@/app/components/tree";
import { datasetTypes } from "@/app/_util/datasets";
import { attrTypes } from "@/app/_util/attrs";
import { useDeleteAttrModal } from "./modals/delattr";
import { useAddAttrModal } from "./modals/addattr";
import { useDeleteClassModal } from "./modals/delclass";
import { useAddClassModal } from "./modals/addclass";
import { useDeleteDatasetModal } from "./modals/delds";

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

		fetch("/api/dataset/list")
			.then((res) => res.json())
			.then((data: { ds_type: string; name: string }[]) => {
				return Promise.all(
					data.map(async ({ ds_type, name: dataset }) => {
						const res = await fetch(
							"/api/class/list?" +
								new URLSearchParams({
									dataset,
								}).toString(),
						);
						const data: {
							name: string;
							attrs: { name: string; data_type: { type: string } }[];
						}[] = await res.json();

						return {
							name: dataset,
							type: ds_type,
							classes: data.map((x) => {
								return {
									name: x.name,
									attrs: x.attrs.map((y) => {
										return {
											name: y.name,
											type: y.data_type.type,
										};
									}),
								};
							}),
						};
					}),
				);
			})
			.then((data) => {
				console.log(data);

				const tree_data: TreeNode<null>[] = [];
				for (let di = 0; di < data.length; di++) {
					const d = data[di];
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

					for (let ci = 0; ci < d.classes.length; ci++) {
						const c = d.classes[ci];
						const c_node = tree_data.push({
							icon: <XIconFolder />,
							text: c.name,
							right: (
								<ClassMenu
									dataset_name={d.name}
									class_name={c.name}
									onSuccess={update_tree}
								/>
							),
							selectable: false,
							uid: `dataset-${d.name}-class-${c.name}`,
							parent: d_node - 1,
							can_have_children: true,
							data: null,
						});

						for (let ai = 0; ai < c.attrs.length; ai++) {
							const a = c.attrs[ai];
							let a_type = attrTypes.find((x) => x.serialize_as === a.type);
							tree_data.push({
								icon: a_type?.icon,
								text: a.name,
								right: (
									<AttrMenu
										dataset_name={d.name}
										class_name={c.name}
										attr_name={a.name}
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
				<XIconX
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
				<XIconDatabaseX
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
				panel_id={styles.panel_tree}
				icon={<XIconDatabase />}
				title={"Manage datasets"}
			>
				<PanelTitle icon={<XIconSettings />} title={"Control Panel"} />
				<Button
					radius="0"
					onClick={() => {
						openModal();
					}}
					variant="light"
					color="green"
					fullWidth
					leftSection={<XIconDatabasePlus />}
					style={{ cursor: "default" }}
				>
					Create a new dataset
				</Button>

				<PanelTitle icon={<XIconDatabase />} title={"Datasets"} />
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

function ClassMenu(params: {
	dataset_name: string;
	class_name: string;
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
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
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

function AttrMenu(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
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
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
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
