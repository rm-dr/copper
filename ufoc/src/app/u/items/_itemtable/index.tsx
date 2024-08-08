import styles from "./itemtable.module.scss";
import { Panel } from "@/app/components/panel";
import clsx from "clsx";
import { ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { Loader, Text } from "@mantine/core";
import { ColumnHeader } from "./parts/columnheader";
import { ItemData, Selected } from "../page";
import { attrTypes } from "@/app/_util/attrs";
import {
	IconCircleOff,
	IconDatabaseX,
	IconFolderX,
	IconListDetails,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";

export function ItemTablePanel(params: { data: ItemData; select: Selected }) {
	return (
		<Panel
			panel_id={styles.panel_itemtable}
			icon={<XIcon icon={IconListDetails} />}
			title={"Item Table"}
		>
			<ItemTable data={params.data} select={params.select} minCellWidth={120} />
		</Panel>
	);
}

const TablePlaceholder = (params: { children: ReactNode }) => {
	return (
		<tr>
			<td>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						alignItems: "center",
						marginTop: "4rem",
						marginBottom: "4rem",
						color: "var(--mantine-color-dimmed)",
					}}
				>
					{params.children}
				</div>
			</td>
		</tr>
	);
};

// TODO: delete top of list once we scroll too far
// (save memory for large lists)

const ItemTable = (params: {
	data: ItemData;
	select: Selected;

	// Minimal cell width, in px
	minCellWidth: number;
}) => {
	const tableRootElement = useRef<any>(null);
	const columnUidCounter = useRef(1);
	const [columns, setColumns] = useState<
		{
			attr: null | number;
			unique_id: number;
		}[]
	>(
		// Start with one unset column.
		// Note that our grid def in .scss also defines one column.
		[{ unique_id: 0, attr: null }],
	);

	// Reset table when dataset or class changes
	useEffect(() => {
		columnUidCounter.current = 1;
		setColumns([{ unique_id: 0, attr: null }]);
		tableRootElement.current.style.gridTemplateColumns = "1fr";
	}, [params.data.class, params.data.dataset]);

	const updateData = useCallback(() => {
		const e = tableRootElement.current;
		if (e === null) {
			return;
		}

		const isScrollable = e.scrollHeight > e.clientHeight;

		const isScrolledToBottom =
			e.scrollHeight < e.clientHeight + e.scrollTop + 1;

		const isScrolledToTop = isScrolledToBottom ? false : e.scrollTop === 0;

		if (isScrolledToBottom) {
			params.data.loadMore();
		}
	}, [tableRootElement, params.data]);

	const col_refs = useRef<(HTMLTableCellElement | null)[]>([null, null]);

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
	// Table body
	//

	let table_body;
	if (params.data.dataset === null) {
		table_body = (
			<TablePlaceholder>
				<XIcon icon={IconDatabaseX} style={{ height: "6rem" }} />
				<div>
					<Text size="1.5rem">No dataset selected</Text>
				</div>
			</TablePlaceholder>
		);
	} else if (params.data.class === null) {
		table_body = (
			<TablePlaceholder>
				<XIcon icon={IconFolderX} style={{ height: "6rem" }} />
				<div>
					<Text size="1.5rem">No class selected</Text>
				</div>
			</TablePlaceholder>
		);
	} else if (params.data.data.length === 0 && params.data.loading) {
		table_body = (
			<TablePlaceholder>
				<Loader color="var(--mantine-color-dimmed)" size="4rem" />
				<div>
					<Text size="1.5rem">Loading...</Text>
				</div>
			</TablePlaceholder>
		);
	} else if (params.data.data.length === 0) {
		table_body = (
			<TablePlaceholder>
				<XIcon icon={IconCircleOff} style={{ height: "6rem" }} />
				<div>
					<Text size="1.5rem">No items in this class</Text>
				</div>
			</TablePlaceholder>
		);
	} else {
		table_body = params.data.data.map((data_entry, data_idx) => {
			if (data_entry.attrs === undefined) {
				return null;
			}

			const selected = params.select.selected.includes(data_idx);
			return (
				<tr
					key={data_entry.idx}
					className={clsx(styles.itemdata, selected && styles.selected)}
					onClick={(c) => {
						if (c.ctrlKey) {
							if (selected) {
								params.select.deselect(data_idx);
							} else {
								params.select.select(data_idx);
							}
						} else if (c.shiftKey) {
							params.select.select_through(data_idx);
						} else {
							params.select.clear();
							params.select.select(data_idx);
						}
					}}
				>
					{columns.map(({ attr }, c_idx) => {
						if (attr === null) {
							return (
								<td key={`${data_entry.idx}-${c_idx}-${attr}`}>
									<Text c="dimmed" fs="italic">
										no attribute
									</Text>
								</td>
							);
						}

						// type hack (TODO: fix)
						let found_attr = Object.entries(data_entry.attrs).find(
							([_, x]) => x?.attr.handle === attr,
						);
						let found_attr_x =
							found_attr === undefined ? undefined : found_attr[1];

						const d = attrTypes.find((x) => {
							return x.serialize_as === found_attr_x?.type;
						});
						if (d === undefined) {
							return (
								<td key={`${data_entry.idx}-${c_idx}-${attr}`}>
									<Text c="dimmed" fs="italic">
										invalid attr type
									</Text>
								</td>
							);
						}

						let v = data_entry.attrs[attr.toString()];
						return (
							<td key={`${data_entry.idx}-${c_idx}-${attr}`}>
								{d.value_preview === undefined || v === undefined
									? null
									: d.value_preview({
											attr_value: v,
									  })}
							</td>
						);
					})}
				</tr>
			);
		});
	}

	//
	// Content
	//

	return (
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
							key={`${params.data.class}-${unique_id}`}
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
									onMouseDown={(e) => {
										if (e.button === 0) {
											resizeStart(idx - 1);
										}
									}}
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
								selectedClass={params.data.class}
								selectedDataset={params.data.dataset}
							/>
						</th>
					))}
				</tr>
			</thead>
			<tbody>
				{table_body}
				{!(params.data.loading && params.data.data.length !== 0) ? null : (
					<tr>
						<td>
							<div
								style={{
									display: "flex",
									flexDirection: "column",
									alignItems: "center",
									marginTop: "1rem",
									marginBottom: "1rem",
									color: "var(--mantine-color-dimmed)",
								}}
							>
								<Loader color="var(--mantine-color-dimmed)" size="2rem" />
							</div>
						</td>
					</tr>
				)}
			</tbody>
		</table>
	);
};
