"use client";

import styles from "./page.module.scss";
import { DatsetPanel } from "./_datasets";
import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";
import { ItemTablePanel } from "./_itemtable";
import { EditPanel } from "./_edit";

export default function Page() {
	const [selectedDataset, setSelectedDataset] = useState<string | null>(null);
	const [selectedClass, setSelectedClass] = useState<string | null>(null);

	const select = useSelected();

	const { itemdata, resetitemdata } = useItemData({
		dataset: selectedDataset,
		class: selectedClass,
	});

	return (
		<main className={styles.main}>
			<div className={styles.wrap_top}>
				<div className={styles.wrap_list}>
					<ItemTablePanel data={itemdata} select={select} />
				</div>
				<div className={styles.wrap_right}>
					<DatsetPanel
						selectedDataset={selectedDataset}
						setSelectedDataset={(v) => {
							resetitemdata();
							setSelectedDataset(v);
							select.clear();
						}}
						setSelectedClass={(v) => {
							resetitemdata();
							setSelectedClass(v);
							select.clear();
						}}
					/>
				</div>
			</div>
			<div className={styles.wrap_bottom}>
				<EditPanel data={itemdata} select={select} />
			</div>
		</main>
	);
}

const PAGE_SIZE = 15;

async function fetchdata(params: {
	class: string | null;
	dataset: string | null;
	maxPage: number;

	setLoading: (loading: boolean) => void;
	setData: Dispatch<SetStateAction<{}[]>>;
}) {
	// TODO: data isn't loaded if more than PAGE_SIZE items fit on the screen
	params.setLoading(true);
	if (params.class === null || params.dataset === null) {
		params.setData([]);
		return;
	}

	for (let page = 0; page <= params.maxPage; page++) {
		const res = await fetch(
			"/api/item/list?" +
				new URLSearchParams({
					dataset: params.dataset,
					class: params.class,
					page_size: PAGE_SIZE.toString(),
					start_at: (page * PAGE_SIZE).toString(),
				}).toString(),
		);
		const json = await res.json();
		params.setData((d) => [
			...d.slice(0, page * PAGE_SIZE),
			...json.items,
			...d.slice(page * PAGE_SIZE + PAGE_SIZE),
		]);
	}
	params.setLoading(false);
}

export type ItemData = {
	dataset: string | null;
	class: string | null;
	loading: boolean;
	data: any[];
	loadMore: () => void;
};

function useItemData(params: { dataset: string | null; class: string | null }) {
	const [loading, setLoading] = useState(true);
	const [data, setData] = useState<{}[]>([]);
	const [maxPage, setMaxPage] = useState(0);

	useEffect(() => {
		fetchdata({
			dataset: params.dataset,
			class: params.class,
			maxPage,
			setData,
			setLoading,
		});
	}, [params.dataset, params.class, maxPage]);

	return {
		resetitemdata: () => {
			setMaxPage(0);
			setData([]);
		},
		itemdata: {
			dataset: params.dataset,
			class: params.class,
			loading,
			data,
			loadMore: () => {
				setMaxPage(Math.ceil(data.length / PAGE_SIZE) + 1);
			},
		},
	};
}

export type Selected = {
	selected: number[];
	select: (idx: number) => void;
	select_through: (idx: number) => void;
	deselect: (idx: number) => void;
	clear: () => void;
};

function useSelected(): Selected {
	const [selectedItems, setSelectedItems] = useState<number[]>([]);
	const last_selected = useRef<null | number>(null);

	return {
		selected: selectedItems,

		select: (v: number) => {
			last_selected.current = v;
			setSelectedItems((s) => [...s, v]);
		},

		select_through: (v: number) => {
			if (last_selected.current === null) {
				setSelectedItems((s) => [...s, v]);
			} else {
				const a = Math.min(v, last_selected.current);
				const b = Math.max(v, last_selected.current);
				let out: number[] = [];
				for (let i = a; i <= b; i++) {
					out = [...out, i];
				}
				setSelectedItems((s) => [...s, ...out]);
			}
			last_selected.current = v;
		},

		deselect: (v: number) => {
			last_selected.current = null;
			setSelectedItems((s) => {
				let idx = s.findIndex((x) => x === v);
				if (idx === undefined) {
					return s;
				} else {
					return [...s.slice(0, idx), ...s.slice(idx + 1)];
				}
			});
		},

		clear: () => {
			setSelectedItems([]);
		},
	};
}
