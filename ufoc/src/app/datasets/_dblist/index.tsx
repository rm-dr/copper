import styles from "./dblist.module.scss";
import { Panel, PanelSection } from "../../components/panel";

import {
	XIconDatabase,
	XIconDatabasePlus,
	XIconEdit,
	XIconServer,
	XIconSettings,
	XIconTrash,
} from "@/app/components/icons";
import {
	ActionIcon,
	Button,
	Modal,
	Select,
	Text,
	TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useEffect, useState } from "react";
import { Slider } from "../_util/slider";
import { DeleteDatasetButton } from "./parts/del_dataset";

type DatasetList = {
	pipelines: {
		name: string;
		ds_type: string;
		selected: boolean;
	}[];
	error: boolean;
};

export function useDbList(
	select_dataset: (dataset_name: string | null) => void,
	selected_dataset: string | null,
) {
	const [datasetList, setDatasetList] = useState<DatasetList>({
		pipelines: [],
		error: false,
	});

	const update_ds_list = (
		select_dataset: (dataset_name: string | null) => void,
	) => {
		select_dataset(null);
		fetch("/api/dataset/list")
			.then((res) => res.json())
			.then((data) => {
				setDatasetList({
					pipelines: data,
					error: false,
				});
			})
			.catch(() => {
				setDatasetList({
					pipelines: [],
					error: true,
				});
			});
	};

	useEffect(() => {
		update_ds_list(select_dataset);
	}, [select_dataset]);

	const { open: openModal, modal: newDsModal } = useNewDsModal(() => {
		update_ds_list(select_dataset);
	});

	return (
		<>
			{newDsModal}
			<Panel
				panel_id={styles.panel_id_list}
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
					<div className={styles.dataset_list}>
						{datasetList.pipelines.map(({ name, ds_type }, idx) => {
							const is_selected = name == selected_dataset;
							return (
								<div
									key={`dataset-${name}`}
									style={{
										paddingLeft: is_selected ? "2rem" : "0",
										transition: "200ms",
									}}
								>
									<Slider
										is_clickable={true}
										is_selected={is_selected}
										onClick={() => {
											if (is_selected) {
												select_dataset(null);
											} else {
												select_dataset(name);
											}
										}}
										icon={<XIconServer />}
										icon_text={ds_type}
										text={name}
										right={
											<>
												{/*
												<ActionIcon
													variant="light"
													aria-label="Rename this dataset"
													color="blue"
													disabled={!is_selected}
													onMouseDown={console.log}
												>
													<XIconEdit style={{ width: "70%", height: "70%" }} />
												</ActionIcon>
												*/}
												<ActionIcon
													variant="light"
													aria-label="Delete this dataset"
													color="red"
													disabled={!is_selected}
													onMouseDown={console.log}
												>
													<XIconTrash style={{ width: "70%", height: "70%" }} />
												</ActionIcon>
											</>
										}
									/>
								</div>
							);
						})}
					</div>
				</PanelSection>
			</Panel>
		</>
	);
}

export function useNewDsModal(onSuccess: () => void) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [errorReason, setErrorReason] = useState<string | null>(null);

	const [newDsName, setNewDsName] = useState("");
	const [newDsType, setNewDsType] = useState<null | string>(null);

	return {
		open,
		modal: (
			<Modal
				opened={opened}
				onClose={() => {
					if (!isLoading) {
						close();
					}
				}}
				title="Create a new dataset"
				//size="50rem"
				centered
				overlayProps={{
					backgroundOpacity: 0.5,
					blur: 1,
				}}
			>
				<TextInput
					placeholder="dataset name..."
					required={true}
					disabled={isLoading}
					error={errorReason == "name"}
					onChange={(e) => {
						if (errorReason == "name") {
							setErrorReason(null);
							setErrorMessage(null);
						}
						setNewDsName(e.currentTarget.value);
					}}
				/>
				<Select
					required={true}
					style={{ marginTop: "1rem" }}
					placeholder={"select dataset type..."}
					data={["LocalDataset"]}
					error={errorReason == "type"}
					onChange={(value, _option) => {
						if (errorReason == "type") {
							setErrorReason(null);
							setErrorMessage(null);
						}
						setNewDsType(value);
					}}
					disabled={isLoading}
					comboboxProps={{
						transitionProps: { transition: "fade-down", duration: 200 },
					}}
					clearable
				/>
				<Button.Group style={{ marginTop: "1rem" }}>
					<Button
						variant="light"
						fullWidth
						color="red"
						onMouseDown={close}
						disabled={isLoading}
					>
						Cancel
					</Button>
					<Button
						variant="filled"
						fullWidth
						color="green"
						loading={isLoading}
						onMouseDown={() => {
							setLoading(true);
							if (newDsName == "" || newDsName === null) {
								setLoading(false);
								setErrorReason("name");
								setErrorMessage("Name cannot be empty");
								return;
							} else if (newDsType === null) {
								setLoading(false);
								setErrorReason("type");
								setErrorMessage("This field is required");
								return;
							}

							setErrorReason(null);
							setErrorMessage(null);

							fetch(`/api/dataset/add`, {
								method: "POST",
								headers: {
									"Content-Type": "application/json",
								},
								body: JSON.stringify({
									name: newDsName,
									params: {
										type: newDsType,
									},
								}),
							}).then((res) => {
								setLoading(false);
								if (res.status == 400) {
									res.text().then((text) => {
										setErrorMessage(text);
									});
								} else if (!res.ok) {
									res.text().then((text) => {
										setErrorMessage(`Error ${res.status}: ${text}`);
									});
								} else {
									// Successfully created new dataset
									onSuccess();
									close();
								}
							});
						}}
					>
						Create
					</Button>
				</Button.Group>
				{errorMessage !== null ? (
					<div
						style={{
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
						}}
					>
						<Text c="red">{errorMessage}</Text>
					</div>
				) : (
					<></>
				)}
			</Modal>
		),
	};
}
