"use client";

import styles from "./page.module.scss";
import { DatasetPanel } from "./_datasets";
import { useCallback, useRef, useState } from "react";
import { ItemTablePanel } from "./_itemtable";
import { EditPanel } from "./_edit";
import { components } from "@/app/_util/api/openapi";
import { APIclient } from "@/app/_util/api";

const PAGE_SIZE = 50;

export type ItemData = {
	loading: boolean;
	data: components["schemas"]["ItemListItem"][];
};

export type selectedClass =
	| { dataset: null; class_idx: null; attrs: null }
	| {
		dataset: string;
		class_idx: null;
		attrs: null;
	}
	| {
		dataset: string;
		class_idx: number;
		attrs: components["schemas"]["AttrInfo"][];
	};

export default function Page() {
	const [selectedClass, setSelectedClass] = useState<selectedClass>({
		dataset: null,
		class_idx: null,
		attrs: null,
	});

	const select = useSelected();

	const [itemdata, setItemData] = useState<ItemData>({
		loading: false,
		data: [],
	});

	const load_items = useCallback(
		(maxPage: number, sel: Omit<selectedClass, "attrs">) => {
			if (sel.dataset === null || sel.class_idx === null) {
				return { data: [], attrs: [] };
			}

			setItemData((i) => ({
				...i,
				loading: true,
			}));

			fetchdata({
				dataset: sel.dataset,
				class: sel.class_idx,
				maxPage,
			}).then(({ data }) => {
				setItemData({
					loading: false,
					data,
				});
			});
		},
		[],
	);

	return (
		<main className={styles.main}>
			<div className={styles.wrap_top}>
				<div className={styles.wrap_list}>
					<ItemTablePanel
						key={`${selectedClass.dataset}-${selectedClass.class_idx}`}
						sel={selectedClass}
						select={select}
						data={itemdata}
						minCellWidth={120}
						load_more_items={() => {
							load_items(
								Math.ceil(itemdata.data.length / PAGE_SIZE) + 1,
								selectedClass,
							);
						}}
					/>
				</div>

				<div className={styles.wrap_right}>
					<DatasetPanel
						dataset={selectedClass.dataset}
						class={(
							v:
								| {
									dataset: string;
									class_idx: number | null;
								}
								| { dataset: null; class_idx: null },
						) => {
							if (v.class_idx !== null) {
								const c = v.class_idx;
								const d = v.dataset;
								APIclient.GET("/class/get", {
									params: {
										query: {
											dataset: d,
											class: c,
										},
									},
								}).then(({ data, error }) => {
									if (error !== undefined) {
										throw error;
									}

									setSelectedClass({
										dataset: d,
										class_idx: c,
										attrs: data.attrs,
									});
								});
							} else {
								setSelectedClass({
									dataset: v.dataset,
									class_idx: v.class_idx,
									attrs: null,
								});
							}

							load_items(0, v);
							select.clear();
						}}
					/>
				</div>
			</div>

			<div className={styles.wrap_bottom}>
				<EditPanel
					key={`${selectedClass.dataset}-${selectedClass.class_idx}`}
					sel={selectedClass}
					select={select}
					data={itemdata}
				/>
			</div>
		</main>
	);
}

async function fetchdata(params: {
	class: number;
	dataset: string;
	maxPage: number;
}) {
	// TODO: data isn't loaded if more than PAGE_SIZE items fit on the screen

	let d: components["schemas"]["ItemListItem"][] = [];
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
			throw error;
		} else {
			d = [...d, ...data.items];
		}
	}

	return {
		data: d,
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
