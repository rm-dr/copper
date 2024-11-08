"use client";

import TitleBar from "@/components/titlebar";
import mainStyle from "../items.module.scss";
import tableStyle from "./table.module.scss";
import { keepPreviousData, useInfiniteQuery } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import {
	Dispatch,
	SetStateAction,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from "react";
import { Button, Checkbox, Popover, Select, Text } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { getAttrTypeInfo } from "@/lib/attributes";
import { Plus, Trash2, TriangleAlert, UnfoldHorizontal, X } from "lucide-react";
import {
	ColumnDef,
	flexRender,
	getCoreRowModel,
	Header,
	RowSelectionState,
	useReactTable,
} from "@tanstack/react-table";
import { Spinner, Wrapper } from "@/components/spinner";

const fetchSize = 50;

/**
 * Generate a react-table column for the given attribute
 */
function gen_attr_col(
	attr: components["schemas"]["AttributeInfo"],
): ColumnDef<components["schemas"]["ItemlistItemInfo"]> {
	return {
		// Column parameters
		id: `attr-${attr.id}`,
		header: attr.name,
		minSize: 150,

		// Row data accessor
		accessorFn: (row) => {
			const val = row.attribute_values[attr.id.toString()];

			if (val === undefined) {
				console.error("Entered unreachable code: cell `val` is undefined");
				throw new Error("Entered unreachable code: cell `val` is undefined");
			}

			val;
		},

		// Cell render fn
		cell: ({ row }) => {
			const value = row.original.attribute_values[attr.id.toString()] || null;

			if (!(value === null || value.type === attr.data_type.type)) {
				throw new Error(
					`Attribute type mismatch: ${value.type} and ${attr.data_type.type}`,
				);
			}

			const attrdef = getAttrTypeInfo(attr.data_type.type);

			let jsx = null;
			if (value === null) {
				jsx = (
					<div
						style={{
							color: "var(--mantine-color-dimmed)",
							paddingLeft: "0.5rem",
							fontStyle: "italic",
						}}
					>
						No value
					</div>
				);
			} else {
				jsx = attrdef.table_cell(value);
			}

			if (jsx === null) {
				console.error("Entered unreachable code: cell jsx is undefined");
				throw new Error("Entered unreachable code: cell jsx is undefined");
			}

			return jsx;
		},
	};
}

function initialize_columns(item_class: components["schemas"]["ClassInfo"]) {
	const init: ColumnDef<components["schemas"]["ItemlistItemInfo"]>[] = [
		// Selection column
		{
			accessorFn: (row) => {
				return row.id;
			},
			id: "select",
			size: 20,
			enableResizing: false,

			header: ({ table }) => (
				<div
					style={{
						display: "flex",
						flexDirection: "row",
						alignItems: "center",
						justifyContent: "center",
						width: "100%",
					}}
				>
					<Checkbox
						checked={table.getIsAllRowsSelected()}
						indeterminate={table.getIsSomeRowsSelected()}
						onChange={() => {
							table.setRowSelection({});
						}}
					/>
				</div>
			),

			cell: ({ row, table }) => (
				<div
					style={{
						display: "flex",
						flexDirection: "row",
						alignItems: "center",
						justifyContent: "center",
						width: "100%",
					}}
				>
					<Checkbox
						readOnly={true}
						checked={row.getIsSelected()}
						disabled={!row.getCanSelect()}
						indeterminate={row.getIsSomeSelected()}
						onClick={(event) => {
							if (event.ctrlKey) {
								row.getToggleSelectedHandler()(event);
							} else {
								table.setRowSelection({});
								row.getToggleSelectedHandler()(event);
							}
						}}
					/>
				</div>
			),
		},

		// ID column
		{
			accessorFn: (row) => {
				return row.id;
			},
			id: "id",
			header: "id",
			size: 75,
			enableResizing: false,

			cell: ({ row }) => (
				<div
					style={{
						width: "100%",
						textAlign: "center",
						fontFamily: "monospace",
						color: "var(--mantine-color-dark-5)",
					}}
				>
					{row.id}
				</div>
			),
		},
	];

	// Initialize with at most this many attribute columns
	let max_cols = 2;
	for (const attr of item_class.attributes) {
		if (max_cols === 0) break;
		max_cols -= 1;

		init.push(gen_attr_col(attr));
	}

	return init;
}

function useItemQuery(item_class: components["schemas"]["ClassInfo"] | null) {
	return useInfiniteQuery({
		queryKey: ["class/items", item_class?.id],
		queryFn: async ({ pageParam = 0 }) => {
			if (item_class === null) {
				return [];
			}

			const res = await edgeclient.GET("/class/{class_id}/items", {
				params: {
					path: {
						class_id: item_class.id,
					},
					query: {
						skip: pageParam * fetchSize,
						count: fetchSize,
					},
				},
			});

			if (res.response.status === 401) {
				location.replace("/");
			}

			if (res.response.status !== 200) {
				throw new Error("could not get items");
			}

			return res.data!;
		},
		initialPageParam: 0,
		getNextPageParam: (_lastGroup, groups) => groups.length,
		refetchOnWindowFocus: false,
		placeholderData: keepPreviousData,
	});
}

function TableHeader(params: {
	item_class: components["schemas"]["ClassInfo"];

	header: Header<
		{
			attribute_values: {
				[key: string]: components["schemas"]["ItemAttrData"];
			};
			class: number;
			id: number;
		},
		unknown
	>;

	columns: ColumnDef<{
		attribute_values: {
			[key: string]: components["schemas"]["ItemAttrData"];
		};
		class: number;
		id: number;
	}>[];

	setColumns: Dispatch<
		SetStateAction<
			ColumnDef<{
				attribute_values: {
					[key: string]: components["schemas"]["ItemAttrData"];
				};
				class: number;
				id: number;
			}>[]
		>
	>;
}) {
	const [selectedAttr, setSelectedAttr] = useState<number | null>(null);
	const [opened, setOpened] = useState(false);

	return (
		<div
			className={tableStyle.th}
			style={{
				width: params.header.getSize(),
				overflow: "hidden",
			}}
		>
			<div
				style={{
					display: "flex",
					flexDirection: "row",
				}}
			>
				<div
					style={{
						flexGrow: 1,
						overflow: "hidden",
						textOverflow: "ellipsis",
					}}
				>
					{params.header.isPlaceholder
						? null
						: flexRender(
								params.header.column.columnDef.header,
								params.header.getContext(),
							)}
				</div>

				{
					//
					// Add column
					//
					params.header.column.id === "select" ? null : (
						<div
							style={{
								display: "flex",
								flexDirection: "row",
								justifyContent: "end",
								gap: "0.5rem",
								// Column min width must be AT LEAST this value, which
								// prevents us from moving the resize handle out of view.
								width: params.header.column.id === "id" ? "40px" : "100px",
							}}
						>
							<Popover
								// Add column
								width={200}
								position="bottom"
								withArrow
								shadow="md"
								opened={opened}
							>
								<Popover.Target>
									<div
										className={`${tableStyle.button} ${tableStyle.delete}`}
										onClick={() => setOpened((o) => !o)}
									>
										<Plus />
									</div>
								</Popover.Target>
								<Popover.Dropdown>
									<Select
										comboboxProps={{ withinPortal: false }}
										style={{ width: "100%" }}
										placeholder={"Select an attribute"}
										value={selectedAttr?.toString()}
										onChange={(value) => {
											if (value === null) {
												setSelectedAttr(null);
												return;
											}

											try {
												setSelectedAttr(parseInt(value));
											} catch {
												setSelectedAttr(null);
											}
										}}
										data={params.item_class.attributes
											.map((a) => ({
												label: a.name,
												value: a.id.toString(),
												disabled:
													params.columns.find(
														(c) => c.id === `attr-${a.id}`,
													) !== undefined,
											}))
											.filter((v) => !v.disabled)}
									/>
									<Button
										style={{ marginTop: "0.5rem" }}
										size="xs"
										fullWidth
										onClick={() => {
											const attr = params.item_class.attributes.find(
												(a) => a.id === selectedAttr,
											);

											if (attr !== undefined) {
												params.setColumns((cols) => [
													...cols,
													gen_attr_col(attr),
												]);
											}

											setOpened(false);
										}}
									>
										Add column
									</Button>

									<Button
										style={{ marginTop: "0.5rem" }}
										size="xs"
										fullWidth
										variant="outline"
										onClick={() => {
											setOpened(false);
										}}
									>
										Cancel
									</Button>
								</Popover.Dropdown>
							</Popover>
							{
								//
								// Delete column
								//
								params.header.column.id === "id" ||
								params.header.column.id === "select" ? null : (
									<div
										onClick={() => {
											params.setColumns((cols) =>
												cols.filter((x) => x.id !== params.header.column.id),
											);
										}}
										className={`${tableStyle.button} ${tableStyle.delete}`}
									>
										<Trash2 />
									</div>
								)
							}
							{
								//
								// Resize column
								//
								params.header.column.id === "id" ||
								params.header.column.id === "select" ? null : (
									<div
										onDoubleClick={() => params.header.column.resetSize()}
										onMouseDown={params.header.getResizeHandler()}
										onTouchStart={params.header.getResizeHandler()}
										className={`${tableStyle.button} ${tableStyle.resizer} ${params.header.column.getIsResizing() ? tableStyle.isResizing : ""}`}
									>
										<UnfoldHorizontal />
									</div>
								)
							}
						</div>
					)
				}
			</div>
		</div>
	);
}

export function ItemTablePanel(params: {
	class: components["schemas"]["ClassInfo"] | null;
	setSelectedItems: (
		items: components["schemas"]["ItemlistItemInfo"][],
	) => void;
}) {
	const [rowSelection, setRowSelection] = useState<RowSelectionState>({});
	const tableContainerRef = useRef<HTMLDivElement>(null);

	const items = useItemQuery(params.class);
	const flatData = useMemo(
		() =>
			items.data?.pages?.flatMap((page) =>
				Array.isArray(page) ? [] : page.items,
			) ?? [],
		[items.data],
	);

	const ssi = params.setSelectedItems;
	useEffect(() => {
		const s = flatData.filter((x) => rowSelection[x.id.toString()] === true);
		ssi(s);
	}, [ssi, rowSelection, flatData]);

	const totalItems = useMemo(() => {
		let total = 0;
		if (items.data !== undefined && items.data.pages.length !== 0) {
			const last = items.data.pages[items.data.pages.length - 1]!;
			total = (last as Exclude<typeof last, never[]>).total;
		}

		return total;
	}, [items.data]);

	// Called on scroll and on mount to fetch data
	// as the user scrolls down
	const totalFetched = flatData.length;
	const fetchMoreOnBottomReached = useCallback(
		(containerRefElement?: HTMLDivElement | null) => {
			if (containerRefElement) {
				const { scrollHeight, scrollTop, clientHeight } = containerRefElement;

				// Fetch once the user has scrolled within 500px of the bottom of the table
				if (
					scrollHeight - scrollTop - clientHeight < 500 &&
					!items.isFetching &&
					totalFetched < totalItems
				) {
					items.fetchNextPage();
				}
			}
		},
		[items, totalFetched, totalItems],
	);

	// check if our table is already scrolled to the bottom,
	// if it is we immediately fetch more data
	useEffect(() => {
		fetchMoreOnBottomReached(tableContainerRef.current);
	}, [fetchMoreOnBottomReached]);

	const [columns, setColumns] = useState(
		params.class === null ? [] : initialize_columns(params.class),
	);

	const table = useReactTable({
		data: flatData || [],
		columns,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.id.toString(),
		onRowSelectionChange: setRowSelection,
		columnResizeMode: "onChange",
		columnResizeDirection: "ltr",
		enableColumnResizing: true,
		enableRowSelection: true,

		state: {
			rowSelection: rowSelection,
		},
	});

	if (items.isError) {
		<div className={mainStyle.panel}>
			<TitleBar text="Items" />
			<div className={mainStyle.panel_content}>
				<Wrapper>
					<TriangleAlert size="3rem" color="var(--mantine-color-red-5)" />
					<Text size="1.3rem" c="red">
						Could not fetch items
					</Text>
				</Wrapper>
			</div>
		</div>;
	} else if (flatData === undefined || params.class === null) {
		return (
			<div className={mainStyle.panel}>
				<TitleBar text="Items" />
				<div className={mainStyle.panel_content}>
					<Wrapper>
						<X size="3rem" color="var(--mantine-color-dimmed)" />
						<Text size="1.3rem" c="dimmed">
							No class selected
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	}

	//
	// MARK: jsx
	//

	return (
		<div className={tableStyle.divTable}>
			{/* Head */}

			<div className={tableStyle.thead}>
				{table.getHeaderGroups().map((headerGroup) => (
					<div key={headerGroup.id} className={tableStyle.tr}>
						{headerGroup.headers.map((header) => (
							<TableHeader
								key={header.id}
								item_class={params.class!}
								header={header}
								columns={columns}
								setColumns={setColumns}
							/>
						))}
					</div>
				))}
			</div>

			{/* Body */}

			<div
				className={tableStyle.tbody}
				onScroll={(e) => fetchMoreOnBottomReached(e.target as HTMLDivElement)}
				ref={tableContainerRef}
			>
				{table.getRowModel().rows.map((row) => (
					<div
						key={row.id}
						className={tableStyle.tr}
						onClick={(event) => {
							if (event.ctrlKey) {
								row.getToggleSelectedHandler()(event);
							} else {
								table.setRowSelection({});
								row.getToggleSelectedHandler()(event);
							}
						}}
					>
						{row.getVisibleCells().map((cell) => (
							<div
								key={cell.id}
								className={tableStyle.td}
								style={{
									width: cell.column.getSize(),
								}}
							>
								{flexRender(cell.column.columnDef.cell, cell.getContext())}
							</div>
						))}
					</div>
				))}
				{totalFetched >= totalItems ? null : (
					<div
						style={{
							display: "flex",
							flexDirection: "column",
							justifyContent: "center",
							alignItems: "center",
							margin: "1rem",
							marginBottom: "2rem",
						}}
					>
						<Spinner size={"2rem"} />
					</div>
				)}
			</div>
		</div>
	);
}
