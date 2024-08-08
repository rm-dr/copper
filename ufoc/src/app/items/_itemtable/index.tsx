import styles from "./itemtable.module.scss";
import { Panel, PanelSection } from "../../components/panel";
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
import { ActionIcon, Menu, rem } from "@mantine/core";
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
						data={td.data}
						headers={td.headers}
						minCellWidth={120}
						selectedClass={params.selectedClass}
						selectedDataset={params.selectedDataset}
					/>
				</PanelSection>
			</Panel>
		</>
	);
}

const td = {
	headers: [
		{ pretty_name: "Artist", key: "Artist" },
		{ pretty_name: "AlbumArtist", key: "AlbumArtist" },
	],
	data: Array(50).fill({
		handle: 0,
		Artist: "a",
		AlbumArtist: "b",
	}),
};

const initHeaders = (headers: any[]) => {
	return headers.map((h: any) => ({
		header: h,
		ref: useRef<null | any>(null),
	}));
};

const ItemTable = (params: {
	selectedDataset: string | null;
	selectedClass: string | null;

	// Column headers, in the order they should be shown
	headers: {
		// The string to show the user
		pretty_name: string;

		// The key in `data` that is shown in this column
		key: string;
	}[];

	// Table data to show, in order
	// each entry is one row.
	data: {}[];

	// Minimal cell width, in px
	minCellWidth: number;
}) => {
	const [areColumnsInitialized, setAreColumnsInitialized] = useState(false);
	const tableRootElement = useRef<any>(null);
	const columns = initHeaders(params.headers);

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
				if (i !== columns.length - 1) {
					total_width += col.ref.current.offsetWidth;
				}
				return col.ref.current.offsetWidth;
			});

			// Resize the column we're dragging.
			// There are columns.length - 1 dragable column seperators.
			for (let i = 0; i < columns.length - 1; i++) {
				const col = columns[i];

				if (i === activeResizeHandle) {
					let new_width = e.clientX - col.ref.current.offsetLeft;
					let width_delta = new_width - col.ref.current.offsetWidth;

					// Clamp to maximum width
					if (
						total_width + width_delta >
						tableRootElement.current.offsetWidth
					) {
						new_width = col.ref.current.offsetWidth;
						width_delta = new_width - col.ref.current.offsetWidth;
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
		tableRootElement.current.style.userSelect = "text";
		setActiveResizeHandle(null);
		removeResizeListeners();
	}, [setActiveResizeHandle, removeResizeListeners]);

	useEffect(() => {
		if (!areColumnsInitialized) {
			tableRootElement.current.style.gridTemplateColumns = params.headers
				.map((_) => "1fr")
				.join(" ");
			setAreColumnsInitialized(true);
		}
	}, [areColumnsInitialized, params.headers]);

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

	const windowResize = useCallback(
		(e: UIEvent) => {
			const width = tableRootElement.current.offsetWidth;
			const innerwidth = tableRootElement.current.scrollWidth;

			// How much width we're overflowing, in px.
			// Since the last column has auto width, this is either zero or negative.
			let width_delta = width - innerwidth;
			console.assert(
				width_delta <= 0,
				"Width delta is more than zero, something is wrong.",
			);

			const gridColumns: number[] = columns.map((col, i) => {
				return col.ref.current.offsetWidth;
			});

			// TODO: make this prettier
			// Trim space from columns, starting from the leftmost one
			for (let i = 0; i < columns.length - 1; i++) {
				const col = columns[i];

				let c_width_delta = width_delta;
				if (col.ref.current.offsetWidth + width_delta < params.minCellWidth) {
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
		[params.minCellWidth, columns],
	);

	useEffect(() => {
		window.addEventListener("resize", windowResize);
		return () => {
			window.removeEventListener("resize", windowResize);
		};
	}, [windowResize]);

	//
	// Content
	//

	return (
		<>
			<table className={styles.itemtable} ref={tableRootElement}>
				<thead>
					<tr>
						{columns.map(({ header, ref }: any, idx: number) => (
							<th ref={ref} key={header.key}>
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
									attr={null}
									setAttr={console.log}
									selectedClass={params.selectedClass}
									selectedDataset={params.selectedDataset}
								/>
							</th>
						))}
					</tr>
				</thead>
				<tbody>
					{params.data.map((data_entry: any, idx: number) => {
						return (
							<tr key={idx}>
								{params.headers.map((header: any) => {
									return (
										<td key={`${idx}-${header.key}`}>
											<span>{data_entry[header.key]}</span>
										</td>
									);
								})}
							</tr>
						);
					})}
				</tbody>
			</table>
		</>
	);
};

function ColumnHeader(params: {
	selectedDataset: string | null;
	selectedClass: string | null;
	attr: null | string;
	setAttr: (attr: string | null) => void;
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
				{params.attr === null ? (
					<AttrSelector
						onSelect={params.setAttr}
						selectedClass={params.selectedClass}
						selectedDataset={params.selectedDataset}
					/>
				) : (
					params.attr
				)}
			</div>

			<div className={styles.menuicon}>
				<ColumnMenu disabled={false} />
			</div>
		</div>
	);
}

function ColumnMenu(params: { disabled: boolean }) {
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
					>
						Add column (left)
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconAddRight style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Add column (right)
					</Menu.Item>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Remove this column
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
