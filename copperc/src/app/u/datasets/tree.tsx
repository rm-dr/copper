import { TreeNode, useTree } from "@/components/tree";
import { edgeclient } from "@/lib/api/client";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
	Database,
	FolderPlus,
	PenBox,
	Plus,
	Table,
	Trash2,
	TriangleAlert,
	X,
} from "lucide-react";
import { ActionIcon, Text, Tooltip } from "@mantine/core";
import { useDeleteDatasetModal } from "./_modals/deletedataset";
import { useRenameDatasetModal } from "./_modals/renamedataset";
import { useAddClassModal } from "./_modals/addclass";
import { useDeleteClassModal } from "./_modals/deleteclass";
import { useRenameClassModal } from "./_modals/renameclass";
import { useDeleteAttributeModal } from "./_modals/deleteattribute";
import { useRenameAttributeModal } from "./_modals/renameattribute";
import { useAddAttributeModal } from "./_modals/addattribute";
import { getAttrTypeInfo } from "@/lib/attributes";
import { Spinner, Wrapper } from "@/components/spinner";

export function useTreePanel() {
	const {
		node: DatasetTree,
		data: treeData,
		setTreeData,
	} = useTree<null>({ defaultOpen: true });

	const qc = useQueryClient();

	const list = useQuery({
		queryKey: ["dataset/list"],

		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/list");
			if (res.response.status === 401) {
				location.replace("/");
			}

			const nodes: TreeNode<null>[] = [];

			if (res.data === undefined) {
				return undefined;
			}

			for (const dataset of res.data) {
				const dataset_idx = nodes.length;
				nodes.push({
					left: <Database />,
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
					body: dataset.name,
					selectable: false,
					uid: `dataset-${dataset.id}-${dataset.name}`,
					parent: null,
					can_have_children: true,
					data: null,
				});

				for (const itemclass of dataset.classes) {
					const itemclass_idx = nodes.length;
					nodes.push({
						left: <Table />,
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
						body: itemclass.name,
						selectable: false,
						uid: `itemclass-${itemclass.id}-${itemclass.name}`,
						parent: dataset_idx,
						can_have_children: true,
						data: null,
					});

					for (const attr of itemclass.attributes) {
						const attr_type = attr.data_type.type;
						let tt_text: string = attr_type;
						let bonus_text: string = attr_type;

						if (attr_type === "Hash") {
							tt_text = `Hash (${attr.data_type.hash_type})`;
							bonus_text = `Hash (${attr.data_type.hash_type})`;
						} else if (attr_type === "Reference") {
							const c = dataset.classes.find(
								(x) =>
									// This check is redundant, but it keeps TS happy.
									attr.data_type.type === "Reference" &&
									x.id === attr.data_type.class,
							)!;
							bonus_text = `Reference (${c.name})`;
						}

						nodes.push({
							left: (
								<>
									<Tooltip
										label={tt_text}
										position="left"
										offset={10}
										color="gray"
									>
										<div
											style={{
												display: "flex",
												flexDirection: "row",
												alignItems: "center",
												justifyContent: "center",
												width: "100%",
												height: "100%",
											}}
										>
											{getAttrTypeInfo(attr.data_type.type).icon}
										</div>
									</Tooltip>
								</>
							),
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
							body: (
								<>
									<div
										style={{
											width: "10rem",
											overflowX: "scroll",
											textWrap: "nowrap",
										}}
									>
										<Text>{attr.name}</Text>
									</div>
									<div
										style={{
											display: "flex",
											flexDirection: "row",
											gap: "0.5rem",
										}}
									>
										{!attr.options.is_not_null ? null : (
											<Text c="yellow" fs="italic" size="sm">
												not null
											</Text>
										)}
										{!attr.options.is_unique ? null : (
											<Text c="cyan" fs="italic" size="sm">
												unique
											</Text>
										)}
										{bonus_text === null ? null : (
											<Text c="dimmed" fs="italic" size="sm">
												{bonus_text}
											</Text>
										)}
									</div>
								</>
							),
							selectable: false,
							uid: `att-${attr.id}-${attr.name}`,
							parent: itemclass_idx,
							can_have_children: false,
							data: null,
						});
					}
				}
			}

			setTreeData(nodes);

			return res.data;
		},
	});

	let tree;
	if (list.isPending) {
		tree = (
			<Wrapper>
				<Spinner />
				<Text size="1.3rem" c="dimmed">
					Loading...
				</Text>
			</Wrapper>
		);
	} else if (list.isError) {
		tree = (
			<Wrapper>
				<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
				<Text size="1.3rem" c="red">
					Could not fetch datasets
				</Text>
			</Wrapper>
		);
	} else if (treeData.length === 0) {
		tree = (
			<Wrapper>
				<X size="3rem" color="var(--mantine-color-dimmed)" />
				<Text size="1.3rem" c="dimmed">
					No datasets
				</Text>
			</Wrapper>
		);
	} else {
		tree = DatasetTree;
	}

	return {
		reload: () => {
			qc.invalidateQueries({ queryKey: ["dataset/list"] });
			list.refetch();
		},
		tree,
	};
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

			<Tooltip label="Edit dataset" color="dark" position="right">
				<ActionIcon
					color="white"
					variant="subtle"
					size={"2rem"}
					onClick={openRename}
				>
					<PenBox size="1.3rem" />
				</ActionIcon>
			</Tooltip>

			<Tooltip label="Add class" color="dark" position="right">
				<ActionIcon
					color="white"
					variant="subtle"
					size={"2rem"}
					onClick={openAddClass}
				>
					<FolderPlus size="1.3rem" />
				</ActionIcon>
			</Tooltip>

			<Tooltip label="Delete dataset" color="dark" position="right">
				<ActionIcon
					color="red"
					variant="subtle"
					size={"2rem"}
					onClick={openDelete}
				>
					<Trash2 size="1.3rem" />
				</ActionIcon>
			</Tooltip>
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

			<Tooltip label="Edit class" color="dark" position="right">
				<ActionIcon
					color="white"
					variant="subtle"
					size={"2rem"}
					onClick={openRename}
				>
					<PenBox size="1.3rem" />
				</ActionIcon>
			</Tooltip>

			<Tooltip label="Add attribute" color="dark" position="right">
				<ActionIcon
					color="white"
					variant="subtle"
					size={"2rem"}
					onClick={openAddAttr}
				>
					<Plus size="1.3rem" />
				</ActionIcon>
			</Tooltip>

			<Tooltip label="Delete class" color="dark" position="right">
				<ActionIcon
					color="red"
					variant="subtle"
					size={"2rem"}
					onClick={openDelete}
				>
					<Trash2 size="1.3rem" />
				</ActionIcon>
			</Tooltip>
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

			<Tooltip label="Edit attribute" color="dark" position="right">
				<ActionIcon
					color="white"
					variant="subtle"
					size={"2rem"}
					onClick={openRename}
				>
					<PenBox size="1.3rem" />
				</ActionIcon>
			</Tooltip>

			<Tooltip label="Delete attribute" color="dark" position="right">
				<ActionIcon
					color="red"
					variant="subtle"
					size={"2rem"}
					onClick={openDelete}
				>
					<Trash2 size="1.3rem" />
				</ActionIcon>
			</Tooltip>
		</>
	);
}
