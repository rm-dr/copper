import styles from "./tree.module.scss";
import { Panel, PanelSection } from "@/app/components/panel";

import {
	XIconDatabase,
	XIconDatabasePlus,
	XIconDatabaseX,
	XIconServer,
	XIconSettings,
	XIconX,
} from "@/app/components/icons";
import { Button, Loader, Select, Text } from "@mantine/core";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { DatasetList } from "./parts/dataset";
import { useNewDsModal } from "./parts/newdsmodal";

export type TreeData = {
	error: boolean;
	loading: boolean;

	datasets:
		| null
		| {
				// Dataset info
				name: string;
				type: string;
				open: boolean;
				classes: {
					// Classes in this dataset
					name: string;
					open: boolean;
					attrs: {
						// Attrs in this class
						name: string;
						type: string;
					}[];
				}[];
		  }[];
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
	const [treeData, setTreeData] = useState<TreeData>({
		datasets: null,
		error: false,
		loading: true,
	});

	const update_tree = useCallback(() => {
		setTreeData((td) => {
			return {
				// Keep old data so we can preserve
				// open state
				...td,
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
							open: false,
							classes: data.map((x) => {
								return {
									name: x.name,
									open: false,
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
				setTreeData((t) => {
					const td = { ...t };
					let d = data.map((x) => {
						// Was this dataset opened in the previous treedata?
						let is_open = false;
						let d_idx: number | undefined = undefined;
						if (td.datasets !== null) {
							d_idx = td.datasets.findIndex((y) => {
								return y.name == x.name;
							});
							console.log(d_idx);
							if (d_idx != -1) {
								is_open = td.datasets[d_idx].open;
							} else {
								d_idx = undefined;
							}
						}

						return {
							...x,
							open: is_open,
							classes: x.classes.map((y) => {
								// Was this class opened in the last treedata?
								let is_open = false;
								if (td.datasets !== null && d_idx !== undefined) {
									let c_idx = td.datasets[d_idx].classes.findIndex((z) => {
										return z.name == y.name;
									});
									if (c_idx !== -1) {
										is_open = td.datasets[d_idx].classes[c_idx].open;
									}
								}

								return { ...y, open: is_open };
							}),
						};
					});

					return {
						datasets: d,
						error: false,
						loading: false,
					};
				});
			})
			.catch(() => {
				setTreeData({
					datasets: null,
					error: true,
					loading: false,
				});
			});
	}, []);

	useEffect(() => {
		update_tree();
	}, [update_tree]);

	const { open: openModal, modal: newDsModal } = useNewDsModal(() => {
		update_tree();
	});

	let tree;
	if (treeData.loading) {
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
	} else if (treeData.error) {
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
	} else if (treeData.datasets === null) {
		tree = (
			<Wrapper>
				<XIconX
					style={{
						height: "5rem",
						color: "var(--mantine-color-red-7)",
					}}
				/>
				<Text size="lg" c="red">
					Error: invalid state
				</Text>
			</Wrapper>
		);
	} else if (treeData.datasets.length === 0) {
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
		tree = (
			<DatasetList
				update_tree={update_tree}
				datasets={treeData.datasets}
				setTreeData={setTreeData}
			/>
		);
	}

	return (
		<>
			{newDsModal}
			<Panel
				panel_id={styles.panel_tree}
				icon={<XIconDatabase />}
				title={"Manage datasets"}
			>
				<PanelSection icon={<XIconSettings />} title={"Control Panel"}>
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
				</PanelSection>

				<PanelSection icon={<XIconDatabase />} title={"Datasets"}>
					{tree}
				</PanelSection>
			</Panel>
		</>
	);
}
