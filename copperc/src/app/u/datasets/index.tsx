import { TreeNode, useTree } from "@/components/tree";
import { edgeclient } from "@/lib/api/client";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
	Database,
	Ellipsis,
	FolderPlus,
	Loader,
	PenBox,
	Plus,
	Table,
	Trash2,
	X,
} from "lucide-react";
import { ReactNode } from "react";
import styles from "./page.module.scss";
import { ActionIcon, Menu, Text } from "@mantine/core";
import { useDeleteDatasetModal } from "./_modals/deletedataset";
import { useRenameDatasetModal } from "./_modals/renamedataset";
import { useAddClassModal } from "./_modals/addclass";
import { useDeleteClassModal } from "./_modals/deleteclass";
import { useRenameClassModal } from "./_modals/renameclass";
import { useDeleteAttributeModal } from "./_modals/deleteattribute";
import { useRenameAttributeModal } from "./_modals/renameattribute";
import { useAddAttributeModal } from "./_modals/addattribute";

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

export function TreePanel() {
	const { node: DatasetTree, data: treeData, setTreeData } = useTree<null>({});

	const qc = useQueryClient();

	const list = useQuery({
		queryKey: ["dataset/list"],
		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/list");
			if (res.response.status !== 200) {
				location.replace("/");
			}

			if (res.data === undefined) {
				return false;
			}

			const nodes: TreeNode<null>[] = [];

			for (const dataset of res.data) {
				const dataset_idx = nodes.length;
				nodes.push({
					icon: <Database />,
					right: (
						<DatasetMenu
							dataset_id={dataset.id}
							dataset_name={dataset.name}
							onSuccess={() => {
								qc.invalidateQueries({ queryKey: ["dataset/list"] });
								list.refetch();
							}}
						/>
					),
					text: dataset.name,
					selectable: false,
					uid: `dataset-${dataset.id}`,
					parent: null,
					can_have_children: true,
					data: null,
				});

				for (const itemclass of dataset.classes) {
					const itemclass_idx = nodes.length;
					nodes.push({
						icon: <Table />,
						right: (
							<ClassMenu
								dataset_id={dataset.id}
								class_id={itemclass.id}
								class_name={itemclass.name}
								onSuccess={() => {
									qc.invalidateQueries({ queryKey: ["dataset/list"] });
									list.refetch();
								}}
							/>
						),
						text: itemclass.name,
						selectable: false,
						uid: `itemclass-${itemclass.id}`,
						parent: dataset_idx,
						can_have_children: true,
						data: null,
					});

					for (const attr of itemclass.attributes) {
						nodes.push({
							icon: <X />,
							right: (
								<AttrMenu
									attribute_id={attr.id}
									attribute_name={attr.name}
									onSuccess={() => {
										qc.invalidateQueries({ queryKey: ["dataset/list"] });
										list.refetch();
									}}
								/>
							),
							text: attr.name,
							selectable: false,
							uid: `att-${attr.id}`,
							parent: itemclass_idx,
							can_have_children: false,
							data: null,
						});
					}
				}
			}

			setTreeData(nodes);

			return true;
		},
	});

	let tree;
	if (list.isPending) {
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
	} else if (list.isError) {
		tree = (
			<Wrapper>
				<X />
				<Text size="lg" c="red">
					Could not fetch datasets
				</Text>
			</Wrapper>
		);
	} else if (treeData.length === 0) {
		tree = (
			<Wrapper>
				<X />
				<Text size="lg" c="dimmed">
					No datasets
				</Text>
			</Wrapper>
		);
	} else {
		tree = DatasetTree;
	}

	return <div className={styles.dataset_list}>{tree}</div>;
}

function DatasetMenu(params: {
	dataset_id: number;
	dataset_name: string;
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteDatasetModal({
		dataset_id: params.dataset_id,
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddClass, modal: modalAddClass } = useAddClassModal({
		dataset_id: params.dataset_id,
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	const { open: openRename, modal: modalRename } = useRenameDatasetModal({
		dataset_id: params.dataset_id,
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddClass}
			{modalRename}
			<Menu
				trigger="click-hover"
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<Ellipsis />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Dataset</Menu.Label>
					<Menu.Item
						leftSection={<PenBox size="1.3rem" />}
						onClick={openRename}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={<FolderPlus size="1.3rem" />}
						onClick={openAddClass}
					>
						Add class
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={<Trash2 size="1.3rem" />}
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
	dataset_id: number;
	class_id: number;
	class_name: string;
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteClassModal({
		class_id: params.class_id,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddAttr, modal: modalAddAttr } = useAddAttributeModal({
		dataset_id: params.dataset_id,
		class_id: params.class_id,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	const { open: openRename, modal: modalRename } = useRenameClassModal({
		class_id: params.class_id,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalRename}
			{modalAddAttr}
			<Menu
				trigger="click-hover"
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<Ellipsis />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Class</Menu.Label>
					<Menu.Item
						leftSection={<PenBox size="1.3rem" />}
						onClick={openRename}
					>
						Rename
					</Menu.Item>
					<Menu.Item leftSection={<Plus size="1.3rem" />} onClick={openAddAttr}>
						Add attribute
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={<Trash2 size="1.3rem" />}
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
	attribute_id: number;
	attribute_name: string;
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteAttributeModal({
		attribute_id: params.attribute_id,
		attribute_name: params.attribute_name,
		onSuccess: params.onSuccess,
	});

	const { open: openRename, modal: modalRename } = useRenameAttributeModal({
		attribute_id: params.attribute_id,
		attribute_name: params.attribute_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalRename}
			{modalDelete}
			<Menu
				trigger="click-hover"
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<Ellipsis />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Attribute</Menu.Label>
					<Menu.Item
						leftSection={<PenBox size="1.3rem" />}
						onClick={openRename}
					>
						Rename
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={<Trash2 size="1.3rem" />}
						onClick={openDelete}
					>
						Delete this attribute
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
