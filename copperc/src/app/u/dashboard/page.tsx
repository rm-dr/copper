"use client";

import TitleBar from "@/components/titlebar";
import styles from "./dashboard.module.scss";
import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { ppBytes } from "@/lib/ppbytes";
import { Fragment } from "react";
import { Text } from "@mantine/core";
import { Spinner, Wrapper } from "@/components/spinner";
import { TriangleAlert } from "lucide-react";

function JobCountEntry(params: {
	title: string;
	count: number;
	color?: string;
}) {
	return (
		<div className={styles.count_line}>
			<div className={styles.count_line_text}>{params.title}</div>
			<div
				className={styles.count_line_count}
				style={{
					width: "33%",
					...(params.color === undefined ? undefined : { color: params.color }),
				}}
			>
				{params.count}
			</div>
		</div>
	);
}

function JobStatusPanel() {
	const jobstatus = useQuery({
		queryKey: ["job/list"],
		refetchInterval: 1000,

		queryFn: async () => {
			const res = await edgeclient.GET("/job/list", {
				params: {
					query: {
						skip: 0,
						count: 100,
					},
				},
			});
			if (res.response.status === 401) {
				location.replace("/");
			}

			if (res.response.status !== 200) {
				throw new Error("could not get jobs");
			}

			return res.data!;
		},
	});

	if (jobstatus.isPending) {
		return (
			<div className={styles.panel}>
				<TitleBar text="Job status" />
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
	} else if (jobstatus.data === undefined) {
		return (
			<div className={styles.panel}>
				<TitleBar text="Job status" />
				<div className={styles.panel_content}>
					<Wrapper>
						<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
						<Text size="1.3rem" c="red">
							Could not fetch jobs
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	}

	return (
		<div className={styles.panel}>
			<TitleBar text="Job status" />
			<div className={styles.panel_content}>
				<JobCountEntry
					title="Queued:"
					count={jobstatus.data.counts.queued_jobs}
				/>

				<JobCountEntry
					title="Running:"
					color="var(--mantine-color-blue-5)"
					count={jobstatus.data.counts.running_jobs}
				/>

				<JobCountEntry
					title="Successful:"
					color="var(--mantine-color-green-5)"
					count={jobstatus.data.counts.successful_jobs}
				/>

				<JobCountEntry
					title="Failed:"
					color="var(--mantine-color-red-5)"
					count={jobstatus.data.counts.failed_jobs}
				/>
			</div>
		</div>
	);
}

function StorageCountEntry(params: {
	title: string;
	count: number;
	is_in_bytes?: boolean;
	color?: string;
}) {
	return (
		<div className={styles.count_line}>
			<div className={styles.count_line_text}>{params.title}</div>
			<div
				className={styles.count_line_count}
				style={{
					width: "7rem",
					...(params.color === undefined ? undefined : { color: params.color }),
				}}
			>
				{params.is_in_bytes ? ppBytes(params.count) : params.count}
			</div>
		</div>
	);
}

function DatasetCountEntry(params: {
	title: string;
	count_items: number;
	padding_left?: string;
	// size_bytes: number;
}) {
	return (
		<div style={{ paddingLeft: params.padding_left, width: "100%" }}>
			<div className={styles.count_line}>
				<div className={styles.count_line_text}>{params.title}</div>
				<div
					className={styles.count_line_count}
					style={{
						width: "11rem",
						textAlign: "center",
						color: "var(--mantine-color-gray-5)",
					}}
				>
					{params.count_items} items
				</div>
				{/*
				<div
					className={styles.count_line_count}
					style={{
						width: "7rem",
					}}
				>
					{ppBytes(params.size_bytes)}
				</div>
				*/}
			</div>
		</div>
	);
}

function StorageStatusPanel() {
	const datasets = useQuery({
		queryKey: ["dataset/list"],
		refetchInterval: 2000,

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

	if (datasets.isPending) {
		return (
			<div className={styles.panel}>
				<TitleBar text="Storage summary" />
				<div className={styles.panel_content}>
					<Wrapper>
						<Spinner />
						<Text size="1.3rem" c="dimmed">
							Loading...
						</Text>
					</Wrapper>
				</div>

				<TitleBar text="Storage by dataset" />
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
			<div className={styles.panel}>
				<TitleBar text="Storage summary" />
				<div className={styles.panel_content}>
					<Wrapper>
						<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
						<Text size="1.3rem" c="red">
							Could not fetch storage summary
						</Text>
					</Wrapper>
				</div>

				<TitleBar text="Storage by dataset" />
				<div className={styles.panel_content}>
					<Wrapper>
						<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
						<Text size="1.3rem" c="red">
							Could not fetch datasets
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	}

	const total_items = datasets.data
		.map((x) => x.classes.reduce((s, c) => s + c.item_count, 0))
		.reduce((s, c) => s + c, 0);

	return (
		<div className={styles.panel}>
			<TitleBar text="Storage summary" />
			<div className={styles.panel_content}>
				<StorageCountEntry title="Total items stored:" count={total_items} />

				{/*
				<StorageCountEntry
					title="Storage used by blobs:"
					is_in_bytes={true}
					count={0}
				/>
				*/}
			</div>

			<TitleBar text="Storage by dataset" />
			<div className={styles.panel_content}>
				{datasets.data.map((x) => {
					const ds_items = x.classes.reduce((s, c) => s + c.item_count, 0);

					return (
						<Fragment key={`dataset-${x.id}`}>
							<DatasetCountEntry
								title={`${x.name}:`}
								count_items={ds_items}
								//size_bytes={0}
							/>
							{x.classes.map((c) => {
								return (
									<DatasetCountEntry
										key={`class-${c.id}`}
										title={`${c.name}:`}
										padding_left="2rem"
										count_items={c.item_count}
										//size_bytes={0}
									/>
								);
							})}
						</Fragment>
					);
				})}
			</div>
		</div>
	);
}

export default function Page() {
	return (
		<>
			<div className={styles.main}>
				<JobStatusPanel />
				<StorageStatusPanel />
			</div>
		</>
	);
}
