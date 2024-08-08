import styles from "./itemtable.module.scss";
import { Panel, PanelSection } from "@/app/components/panel";
import clsx from "clsx";
import {
	XIconAddLeft,
	XIconAddRight,
	XIconDots,
	XIconItems,
	XIconSortDown,
	XIconTrash,
} from "@/app/components/icons";

import { useCallback, useEffect, useRef, useState } from "react";
import { ActionIcon, Code, Menu, Text, rem } from "@mantine/core";
import { AttrSelector } from "@/app/components/apiselect/attr";

export function ItemTablePanel(params: {
	selectedDataset: string | null;
	selectedClass: string | null;
}) {
	return (
		<>
			<Panel
				panel_id={styles.panel_itemtable}
				icon={<XIconItems />}
				title={"Item Table"}
			>
				<PanelSection>
					<ItemTable
						minCellWidth={120}
						selectedClass={params.selectedClass}
						selectedDataset={params.selectedDataset}
					/>
				</PanelSection>
			</Panel>
		</>
	);
}

const ItemTable = (params: {
	selectedDataset: string | null;
	selectedClass: string | null;

	// Minimal cell width, in px
	minCellWidth: number;
}) => {
	const page_size = 50;
	const tableRootElement = useRef<any>(null);
	const columnUidCounter = useRef(1);
	const [columns, setColumns] = useState<
		{
			attr: null | string;
			unique_id: number;
		}[]
	>(
		// Start with one unset column.
		// Note that our grid def in .scss also defines one column.
		[{ unique_id: 0, attr: null }],
	);

	const [data, setData] = useState<{}[]>([]);
	const [dataMaxPage, setDataMaxPage] = useState(0);

	const reset = useEffect(() => {
		columnUidCounter.current = 1;
		setData([]);
		setColumns([{ unique_id: 0, attr: null }]);
		tableRootElement.current.style.gridTemplateColumns = "1fr";
	}, [params]);

	const updateData = useCallback(() => {
		const e = tableRootElement.current;
		if (e == null) {
			return;
		}

		const isScrollable = e.scrollHeight > e.clientHeight;

		const isScrolledToBottom =
			e.scrollHeight < e.clientHeight + e.scrollTop + 1;

		const isScrolledToTop = isScrolledToBottom ? false : e.scrollTop === 0;

		if (isScrolledToBottom) {
			setDataMaxPage(Math.ceil(data.length / page_size) + 1);
		}
	}, [tableRootElement, data]);

	useEffect(() => {
		async function fetchdata() {
			if (params.selectedClass === null || params.selectedDataset === null) {
				return;
			}

			for (let page = 0; page <= dataMaxPage; page++) {
				const res = await fetch(
					"/api/item/list?" +
						new URLSearchParams({
							dataset: params.selectedDataset,
							class: params.selectedClass,
							page_size: page_size.toString(),
							start_at: (page * page_size).toString(),
						}).toString(),
				);
				const json = await res.json();
				setData((d) => [
					...d.slice(0, page * page_size),
					...json.items,
					...d.slice(page * page_size + page_size),
				]);
			}
		}

		fetchdata();
	}, [params.selectedClass, params.selectedDataset, dataMaxPage]);

	const col_refs = useRef<(HTMLTableCellElement | null)[]>([null, null]);

	useEffect(() => {
		setColumns((c) => {
			const n = [...c];
			return n.map((x) => ({
				...x,
				attr: null,
			}));
		});
	}, [params.selectedClass, params.selectedDataset]);

	// The handle we're dragging right now, if any
	const [activeResizeHandle, setActiveResizeHandle] = useState<null | number>(
		null,
	);

	//
	// Column resize logic
	//

	const resizeStart = useCallback((index: number) => {
		setActiveResizeHandle(index);
		tableRootElement.current.style.userSelect = "none";
	}, []);

	const resizeDrag = useCallback(
		(e: MouseEvent) => {
			// The total non-flexible width used by this table.
			//
			// Start by adding the minimum width of the last column.
			// The last column has automatic width, so we don't use it's
			// real with in this calculation.
			let total_width = params.minCellWidth;

			// Initialize with current column widths & compute total
			const gridColumns: number[] = columns.map((col, i) => {
				let cref = col_refs.current[i];

				if (cref === null) {
					console.error(`Col ref ${i} is null!`);

					return 0;
				}
				if (i !== columns.length - 1) {
					total_width += cref.offsetWidth;
				}
				return cref.offsetWidth;
			});

			// Resize the column we're dragging.
			// There are columns.length - 1 dragable column seperators.
			for (let i = 0; i < columns.length - 1; i++) {
				let cref = col_refs.current[i];

				if (cref === null) {
					console.error(`Col ref ${i} is null!`);
					continue;
				}

				if (i === activeResizeHandle) {
					let new_width = e.clientX - cref.offsetLeft;
					let width_delta = new_width - cref.offsetWidth;

					// Clamp to maximum width
					if (
						total_width + width_delta > tableRootElement.current.offsetWidth &&
						// Don't clamp if we're shrinking the column, that should be allowed
						// even if we're somehow too wide.
						!(width_delta < 0)
					) {
						new_width = cref.offsetWidth;
						width_delta = new_width - cref.offsetWidth;
					}

					const new_width_l = gridColumns[i] + width_delta;
					const new_width_r = gridColumns[i + 1] - width_delta;
					if (true) {
						// Resize this column, making space by sliding
						// all columns to the left
						if (new_width_l >= params.minCellWidth) {
							gridColumns[i] = new_width_l;
						}
					} else {
						// Resize this column, making space by
						// changing the width of the column to the right
						if (
							new_width_l >= params.minCellWidth &&
							new_width_r >= params.minCellWidth
						) {
							gridColumns[i] = new_width_l;
							gridColumns[i + 1] = new_width_r;
						}
					}
				}
			}

			tableRootElement.current.style.gridTemplateColumns = gridColumns
				.map((w, i) => {
					if (i === columns.length - 1) {
						return `minmax(${params.minCellWidth}px, auto)`;
					} else {
						return `${w}px`;
					}
				})
				.join(" ");
		},
		[activeResizeHandle, columns, params.minCellWidth],
	);

	const removeResizeListeners = useCallback(() => {
		window.removeEventListener("mousemove", resizeDrag);
		window.removeEventListener("mouseup", removeResizeListeners);
	}, [resizeDrag]);

	const resizeStop = useCallback(() => {
		if (tableRootElement.current !== null) {
			tableRootElement.current.style.userSelect = "text";
		}
		setActiveResizeHandle(null);
		removeResizeListeners();
	}, [setActiveResizeHandle, removeResizeListeners]);

	useEffect(() => {
		if (activeResizeHandle !== null) {
			window.addEventListener("mousemove", resizeDrag);
			window.addEventListener("mouseup", resizeStop);
		}

		return () => {
			removeResizeListeners();
		};
	}, [activeResizeHandle, resizeDrag, resizeStop, removeResizeListeners]);

	//
	// Shrink columns when window is resized
	//

	const shrinkTable = useCallback(
		(by: number) => {
			// How much width we need to shed, in px.
			// Since the last column has auto width, this is either zero or negative.
			let width_delta = -by;
			console.assert(
				width_delta <= 0,
				"Width delta is more than zero, something is wrong.",
			);

			const gridColumns: number[] = columns.map((col, i) => {
				let cref = col_refs.current[i];

				if (cref === null) {
					console.error(`Col ref ${i} is null!`);
					return 0;
				}
				return cref.offsetWidth;
			});

			// TODO: make this prettier
			// Trim space from columns, starting from the leftmost one
			for (let i = 0; i < columns.length - 1; i++) {
				let cref = col_refs.current[i];

				if (cref === null) {
					console.error(`Col ref ${i} is null!`);
					continue;
				}

				let c_width_delta = width_delta;
				if (cref.offsetWidth + width_delta < params.minCellWidth) {
					width_delta += params.minCellWidth - gridColumns[i];
					gridColumns[i] = params.minCellWidth;
				} else {
					width_delta -= c_width_delta;
					gridColumns[i] += c_width_delta;
				}
			}

			tableRootElement.current.style.gridTemplateColumns = gridColumns
				.map((w, i) => {
					if (i === columns.length - 1) {
						return `minmax(${params.minCellWidth}px, auto)`;
					} else {
						return `${w}px`;
					}
				})
				.join(" ");
		},
		[columns, params.minCellWidth],
	);

	const windowResize = useCallback(
		(e: UIEvent) => {
			const width = tableRootElement.current.offsetWidth;
			const innerwidth = tableRootElement.current.scrollWidth;

			// How much width we're overflowing, in px.
			// Since the last column has auto width, this is either zero or positive.
			shrinkTable(innerwidth - width);
		},
		[shrinkTable],
	);

	useEffect(() => {
		window.addEventListener("resize", windowResize);
		return () => {
			window.removeEventListener("resize", windowResize);
		};
	}, [windowResize]);

	//
	// Add columns
	//

	const newColumn = useCallback(
		(idx: number) => {
			// Get old column widths
			const oldGridColumns: number[] = columns.map((_, i) => {
				let cref = col_refs.current[i];

				if (cref === null) {
					console.error(`Col ref ${i} is null!`);
					return 0;
				}

				return cref.offsetWidth;
			});

			// Make new column
			setColumns((c) => {
				columnUidCounter.current += 1;
				return [
					...c.slice(0, idx),
					{ unique_id: columnUidCounter.current, attr: null },
					...c.slice(idx),
				];
			});
			col_refs.current = [
				...col_refs.current.slice(0, idx),
				null,
				...col_refs.current.slice(idx),
			];

			// Make space for new column
			let to_shed = params.minCellWidth + 50;
			const gridColumns: number[] = [];
			for (let i = 0; i < oldGridColumns.length + 1; i++) {
				if (i == idx) {
					gridColumns.push(params.minCellWidth + 50);
				} else if (i > idx) {
					const new_width = Math.max(
						params.minCellWidth,
						oldGridColumns[i - 1] - to_shed,
					);
					to_shed -= oldGridColumns[i - 1] - new_width;
					gridColumns.push(new_width);
				} else {
					const new_width = Math.max(
						params.minCellWidth,
						oldGridColumns[i] - to_shed,
					);
					to_shed -= oldGridColumns[i] - new_width;
					gridColumns.push(new_width);
				}
			}

			tableRootElement.current.style.gridTemplateColumns = gridColumns
				.map((w, i) => {
					if (i === gridColumns.length - 1) {
						return `minmax(${params.minCellWidth}px, auto)`;
					} else {
						return `${w}px`;
					}
				})
				.join(" ");
		},
		[params.minCellWidth, columns],
	);

	const delColumn = useCallback(
		(idx: number) => {
			// We must always have at least one column
			if (columns.length === 1) {
				return;
			}

			// Get old column widths
			const oldGridColumns: number[] = columns.map((_, i) => {
				let cref = col_refs.current[i];
				if (cref === null) {
					console.error(`Col ref ${i} is null!`);
					return 0;
				}

				return cref.offsetWidth;
			});

			// Delete column new column
			setColumns((c) => {
				return [...c.slice(0, idx), ...c.slice(idx + 1)];
			});
			col_refs.current = [
				...col_refs.current.slice(0, idx),
				...col_refs.current.slice(idx + 1),
			];

			// Fix grid layout
			const gridColumns: number[] = [];
			for (let i = 0; i < oldGridColumns.length; i++) {
				if (i > idx) {
					gridColumns.push(oldGridColumns[i]);
				} else if (i !== idx) {
					gridColumns.push(oldGridColumns[i]);
				}
			}

			tableRootElement.current.style.gridTemplateColumns = gridColumns
				.map((w, i) => {
					if (i === gridColumns.length - 1) {
						return `minmax(${params.minCellWidth}px, auto)`;
					} else {
						return `${w}px`;
					}
				})
				.join(" ");
		},
		[params.minCellWidth, columns],
	);

	//
	// Content
	//

	return (
		<div>
			<table
				className={styles.itemtable}
				ref={tableRootElement}
				onScroll={updateData}
			>
				<thead>
					<tr>
						{columns.map(({ attr, unique_id }, idx: number) => (
							<th
								ref={(ref) => {
									col_refs.current[idx] = ref;
								}}
								key={`${params.selectedClass}-${unique_id}`}
							>
								{/*
									Do not show first resize handle.
									Note that each header contains the *previous*
									column's resize bar. This is a z-index hack,
									makes sure that the resize bar goes ON TOP of
									the previous th.
								*/}
								{idx === 0 ? null : (
									<div
										style={{ height: "100%" }}
										onMouseDown={() => resizeStart(idx - 1)}
										className={clsx(
											styles.resize_handle,
											activeResizeHandle === idx - 1 && styles.active,
										)}
									/>
								)}
								<ColumnHeader
									attr={attr}
									idx={idx}
									columns={columns}
									newCol={newColumn}
									delCol={delColumn}
									setAttr={(a) => {
										setColumns((c) => {
											const n = [...c];
											n[idx].attr = a;
											return n;
										});
									}}
									selectedClass={params.selectedClass}
									selectedDataset={params.selectedDataset}
								/>
							</th>
						))}
					</tr>
				</thead>
				<tbody>
					{data.map((data_entry: any) => {
						return (
							<tr key={data_entry.idx}>
								{columns.map(({ attr }, c_idx) => {
									return (
										<td key={`${data_entry.idx}-${c_idx}-${attr}`}>
											<ItemData
												attr={attr === null ? null : data_entry.attrs[attr]}
											/>
										</td>
									);
								})}
							</tr>
						);
					})}
				</tbody>
			</table>
		</div>
	);
};

function ItemData(params: { attr: any | null }) {
	if (params.attr === null) {
		return <Text c="dimmed">No data</Text>;
	}

	if (params.attr.type === "Text") {
		return params.attr.value.length == 0 ? (
			<Text c="dimmed" fs="italic">
				empty string
			</Text>
		) : (
			<Text>{params.attr.value}</Text>
		);
	}

	if (params.attr.type === "None") {
		return (
			<Text c="dimmed" fs="italic">
				Not set
			</Text>
		);
	}

	if (params.attr.type === "Hash") {
		return (
			<Text>
				{`${params.attr.hash_type} hash: `}
				<Code>{params.attr.value}</Code>
			</Text>
		);
	}

	if (params.attr.type === "Reference") {
		return (
			<Text c="dimmed">
				Reference to{" "}
				<Text c="dimmed" fs="italic" span>
					{params.attr.class}
				</Text>
			</Text>
		);
	}

	if (params.attr.type === "Blob") {
		return <Text c="dimmed" fs="italic">{`Blob ${params.attr.handle}`}</Text>;
	}

	return (
		<Text c="dimmed">
			Data with type{" "}
			<Text c="dimmed" fs="italic" span>
				{params.attr.type}
			</Text>
		</Text>
	);
}

function ColumnHeader(params: {
	selectedDataset: string | null;
	selectedClass: string | null;
	attr: null | string;
	idx: number;
	columns: { attr: null | string }[];
	setAttr: (attr: string | null) => void;
	newCol: (at_index: number) => void;
	delCol: (at_index: number) => void;
}) {
	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				justifyContent: "flex-start",
				gap: "0.5rem",
			}}
		>
			<div className={styles.sorticon}>
				<XIconSortDown />
			</div>
			<div>
				<AttrSelector
					onSelect={params.setAttr}
					selectedClass={params.selectedClass}
					selectedDataset={params.selectedDataset}
				/>
			</div>

			<div className={styles.menuicon}>
				<ColumnMenu
					disabled={false}
					newCol={params.newCol}
					delCol={params.delCol}
					idx={params.idx}
					columns={params.columns}
				/>
			</div>
		</div>
	);
}

function ColumnMenu(params: {
	disabled: boolean;
	idx: number;
	columns: { attr: null | string }[];
	newCol: (at_index: number) => void;
	delCol: (at_index: number) => void;
}) {
	return (
		<>
			<Menu
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
				disabled={params.disabled}
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Table Column</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconAddLeft style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={() => {
							params.newCol(params.idx);
						}}
					>
						Add column (left)
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconAddRight style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={() => {
							params.newCol(params.idx + 1);
						}}
					>
						Add column (right)
					</Menu.Item>
					<Menu.Item
						disabled={params.columns.length === 1}
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={() => {
							params.delCol(params.idx);
						}}
					>
						Remove this column
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}