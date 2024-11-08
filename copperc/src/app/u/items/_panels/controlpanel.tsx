"use client";

import TitleBar from "@/components/titlebar";
import styles from "../items.module.scss";
import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { useState } from "react";
import { Select, Text } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { Spinner, Wrapper } from "@/components/spinner";
import { TriangleAlert } from "lucide-react";

export function ControlPanel(params: {
	// This is one-directional.
	// You'll get odd behavior if you change `selectedClass`
	// outside of this component, or if `selectedClass` doesn't
	// initialize as `null`.
	selectedClass: components["schemas"]["ClassInfo"] | null;
	setSelectedClass: (
		new_class: components["schemas"]["ClassInfo"] | null,
	) => void;
}) {
	const [selectedDataset, setSelectedDataset] = useState<number | null>(null);

	const datasets = useQuery({
		queryKey: ["dataset/list"],

		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/list");
			if (res.response.status === 401) {
				location.replace("/");
			}

			if (res.response.status !== 200) {
				throw new Error("could not get datasets");
			}

			return res.data!;
		},
	});

	if (datasets.data === undefined) {
		return (
			<div className={styles.panel} style={{ width: "20rem" }}>
				<TitleBar text="Control panel" />
				<div className={styles.panel_content}>
					<Wrapper>
						<Spinner />

						<Text size="1.3rem" c="dimmed">
							Loading...
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	} else if (datasets.data === undefined) {
		return (
			<div className={styles.panel} style={{ width: "20rem" }}>
				<TitleBar text="Control panel" />
				<div className={styles.panel_content}>
					<Wrapper>
						<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
						<Text size="1.3rem" c="red">
							Could not fetch items
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	}

	const dataset_data =
		datasets.data === undefined
			? []
			: datasets.data.map((x) => {
					return {
						label: x.name,
						value: x.id.toString(),
					};
				});

	const class_data =
		datasets.data === undefined || selectedDataset === null
			? []
			: datasets.data
					.find((x) => x.id === selectedDataset)!
					.classes.map((x) => {
						return {
							label: x.name,
							value: x.id.toString(),
						};
					});

	return (
		<div className={styles.panel} style={{ width: "20rem" }}>
			<TitleBar text="Control panel" />
			<div className={styles.panel_content}>
				<Select
					label="Select dataset"
					style={{ width: "100%" }}
					disabled={datasets.data === undefined}
					placeholder={
						datasets.data === undefined ? "Loading..." : "Select a dataset"
					}
					data={dataset_data}
					value={selectedDataset === null ? null : selectedDataset.toString()}
					onChange={(value) => {
						const int = value === null ? null : parseInt(value);
						if (int === selectedDataset) {
							return;
						}

						if (int === null || datasets.data === undefined) {
							setSelectedDataset(null);
							params.setSelectedClass(null);
							return;
						}

						setSelectedDataset(int);
						params.setSelectedClass(null);
					}}
				/>

				<Select
					label="Select class"
					style={{ width: "100%" }}
					disabled={datasets.data === undefined}
					placeholder={
						datasets.data === undefined ? "Loading..." : "Select a class"
					}
					data={class_data}
					value={
						params.selectedClass === null
							? null
							: params.selectedClass.id.toString()
					}
					onChange={(value) => {
						const int = value === null ? null : parseInt(value);
						if (int === params.selectedClass) {
							return;
						}

						if (int === null || datasets.data === undefined) {
							params.setSelectedClass(null);
							return;
						}

						const c = datasets.data
							.find((x) => x.id === selectedDataset)!
							.classes.find((x) => x.id === int)!;

						params.setSelectedClass(c);
					}}
				/>
			</div>
		</div>
	);
}
