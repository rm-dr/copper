import { components } from "@/app/_util/api/openapi";
import styles from "../itemtable.module.scss";
import { AttrSelector } from "@/app/components/apiselect/attr";
import { XIcon } from "@/app/components/icons";
import { ActionIcon, Menu, Text, rem } from "@mantine/core";
import {
	IconChevronLeftPipe,
	IconChevronRightPipe,
	IconCircleX,
	IconDots,
	IconSortDescending,
	IconTrash,
} from "@tabler/icons-react";

export function ColumnHeader(params: {
	selectedDataset: string | null;
	selectedClass: number | null;
	attr: components["schemas"]["AttrInfo"] | null;
	idx: number;
	n_columns: number;
	setAttr: (attr: number | null) => void;
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
				<XIcon icon={IconSortDescending} />
			</div>
			<div>
				{params.attr === null ? (
					<AttrSelector
						onSelect={params.setAttr}
						selectedClass={params.selectedClass}
						selectedDataset={params.selectedDataset}
					/>
				) : (
					<Text fw={700} size="md">
						{params.attr.name}
					</Text>
				)}
			</div>

			<div className={styles.menuicon}>
				<ColumnMenu
					disabled={false}
					attr={params.attr}
					newCol={params.newCol}
					delCol={params.delCol}
					clearCol={() => params.setAttr(null)}
					idx={params.idx}
					n_columns={params.n_columns}
				/>
			</div>
		</div>
	);
}

function ColumnMenu(params: {
	disabled: boolean;
	idx: number;
	attr: components["schemas"]["AttrInfo"] | null;
	n_columns: number;
	newCol: (at_index: number) => void;
	delCol: (at_index: number) => void;
	clearCol: () => void;
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
						<XIcon icon={IconDots} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Table Column</Menu.Label>

					<Menu.Item
						leftSection={
							<XIcon
								icon={IconCircleX}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={params.clearCol}
						disabled={params.attr === null}
					>
						Clear attribute
					</Menu.Item>

					<Menu.Item
						leftSection={
							<XIcon
								icon={IconChevronLeftPipe}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={() => {
							params.newCol(params.idx);
						}}
					>
						Add column (left)
					</Menu.Item>

					<Menu.Item
						leftSection={
							<XIcon
								icon={IconChevronRightPipe}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={() => {
							params.newCol(params.idx + 1);
						}}
					>
						Add column (right)
					</Menu.Item>

					<Menu.Item
						disabled={params.n_columns === 1}
						color="red"
						leftSection={
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
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
