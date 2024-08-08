"use client";

import styles from "./page.module.scss";
import { DatsetPanel } from "./_datasets";
import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";
import { ItemTablePanel } from "./_itemtable";
import { useEditPanel } from "./_edit";
import { components } from "@/app/_util/api/openapi";
import { APIclient } from "@/app/_util/api";

export default function Page() {
	const [selectedDataset, setSelectedDataset] = useState<string | null>(null);
	const [selectedClass, setSelectedClass] = useState<number | null>(null);

	const select = useSelected();

	const { itemdata, resetitemdata } = useItemData({
		dataset: selectedDataset,
		class: selectedClass,
	});

	const { node: editPanel, on_change_list } = useEditPanel({
		data: itemdata,
		select,
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
							on_change_list();
							select.clear();
						}}
						setSelectedClass={(v) => {
							resetitemdata();
							setSelectedClass(v);
							on_change_list();
							select.clear();
						}}
					/>
				</div>
			</div>
			<div className={styles.wrap_bottom}>{editPanel}</div>
		</main>
	);
}

const PAGE_SIZE = 15;

async function fetchdata(params: {
	class: number | null;
	dataset: string | null;
	maxPage: number;

	setLoading: (loading: boolean) => void;
	setData: Dispatch<SetStateAction<components["schemas"]["ItemListItem"][]>>;
}) {
	// TODO: data isn't loaded if more than PAGE_SIZE items fit on the screen
	params.setLoading(true);
	if (params.class === null || params.dataset === null) {
		params.setData([]);
		return;
	}

	for (let page = 0; page <= params.maxPage; page++) {
		const { data, error } = await APIclient.GET("/item/list", {
			params: {
				query: {
					dataset: params.dataset,
					class: params.class,
					page_size: PAGE_SIZE,
					start_at: page * PAGE_SIZE,
				},
			},
		});

		if (error !== undefined) {
			params.setData([]);
			params.setLoading(false);
			return;
		} else {
			params.setData((d) => [
				...d.slice(0, page * PAGE_SIZE),
				...data.items,
				...d.slice(page * PAGE_SIZE + PAGE_SIZE),
			]);
		}
	}
	params.setLoading(false);
}

export type ItemData = {
	dataset: string | null;
	class: number | null;
	loading: boolean;
	data: components["schemas"]["ItemListItem"][];
	loadMore: () => void;
};

function useItemData(params: { dataset: string | null; class: number | null }) {
	const [loading, setLoading] = useState(true);
	const [data, setData] = useState<components["schemas"]["ItemListItem"][]>([]);
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
