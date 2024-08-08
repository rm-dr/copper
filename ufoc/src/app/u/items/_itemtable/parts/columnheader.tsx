import styles from "../itemtable.module.scss";
import { AttrSelector } from "@/app/components/apiselect/attr";
import { XIcon } from "@/app/components/icons";
import { ActionIcon, Menu, rem } from "@mantine/core";
import {
	IconChevronLeftPipe,
	IconChevronRightPipe,
	IconDots,
	IconSortDescending,
	IconTrash,
} from "@tabler/icons-react";

export function ColumnHeader(params: {
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
				<XIcon icon={IconSortDescending} />
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
						<XIcon icon={IconDots} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Table Column</Menu.Label>
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
						disabled={params.columns.length === 1}
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
