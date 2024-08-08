import styles from "./users.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";

import {
	XIconDots,
	XIconEdit,
	XIconGroup,
	XIconList,
	XIconLock,
	XIconNoItems,
	XIconNoUser,
	XIconSettings,
	XIconTrash,
	XIconUser,
	XIconUserPlus,
	XIconUsers,
} from "@/app/components/icons";
import {
	ActionIcon,
	Button,
	HoverCard,
	Menu,
	Switch,
	Text,
	rem,
} from "@mantine/core";
import { TreeNode } from "@/app/components/tree";
import { GroupData } from "../_grouptree";
import { ReactNode } from "react";

const Wrapper = (params: { children: ReactNode }) => {
	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				width: "100%",
				height: "100%",
				userSelect: "none",
			}}
		>
			<div
				style={{
					display: "block",
					textAlign: "center",
				}}
			>
				{params.children}
			</div>
		</div>
	);
};

export function UsersPanel(params: {
	data: TreeNode<GroupData>[];
	selected: string | null;
}) {
	let g =
		params.selected === null
			? null
			: params.data.find((x) => x.uid === params.selected);

	// This should never happen
	if (g === undefined) {
		return null;
	}

	let userlist = null;
	if (g === null) {
		userlist = (
			<Wrapper>
				<XIconNoItems
					style={{
						height: "5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				/>
				<Text size="lg" c="dimmed">
					No group selected
				</Text>
			</Wrapper>
		);
	} else if (g.data.users.length === 0) {
		userlist = (
			<Wrapper>
				<XIconNoUser
					style={{
						height: "5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				/>
				<Text size="lg" c="dimmed">
					No users in this group
				</Text>
			</Wrapper>
		);
	} else {
		userlist = g?.data.users.map((x) => {
			return (
				<div key={x.id} className={styles.user_entry}>
					<div className={styles.user_entry_icon}>
						<XIconUser />
					</div>
					<div className={styles.user_entry_text}>{x.name}</div>
					<div className={styles.user_entry_right}>
						<UserMenu user_id={x.id} />
					</div>
				</div>
			);
		});
	}

	return (
		<>
			<Panel
				panel_id={styles.panel_users}
				icon={<XIconUsers />}
				title={"Edit group"}
			>
				<PanelTitle icon={<XIconGroup />} title={"Overview"} />
				<div className={styles.overview_container}>
					<div className={styles.overview_entry}>
						<div className={styles.overview_entry_label}>Group:</div>
						<div className={styles.overview_entry_text}>
							{g === null ? (
								<Text c="dimmed">None</Text>
							) : (
								g.data.group_info.name
							)}
						</div>
					</div>
					<div className={styles.overview_entry}>
						<div className={styles.overview_entry_label}>Users:</div>
						<div className={styles.overview_entry_text}>
							{g === null ? <Text c="dimmed">0</Text> : g.data.users.length}
						</div>
					</div>
				</div>

				<PanelTitle icon={<XIconSettings />} title={"Permissions"} />
				<div className={styles.perm_container}>
					<div className={styles.perm_entry}>
						<div className={styles.perm_entry_switch}>
							<Switch defaultChecked size="xs" />
						</div>
						<div className={styles.perm_entry_text}>Edit users</div>
					</div>
					<div className={styles.perm_entry} style={{ marginLeft: "2rem" }}>
						<div className={styles.perm_entry_switch}>
							<Switch defaultChecked size="xs" />
						</div>
						<div className={styles.perm_entry_text}>Edit users</div>
					</div>
				</div>

				<PanelTitle icon={<XIconList />} title={"Manage users"} />
				<Button
					radius="0"
					onClick={() => {}}
					variant="light"
					color="green"
					size="xs"
					fullWidth
					leftSection={<XIconUserPlus />}
					style={{ cursor: "default" }}
				>
					Create a user
				</Button>

				<div className={styles.users_container}>{userlist}</div>
			</Panel>
		</>
	);
}

function UserMenu(params: { user_id: number }) {
	return (
		<>
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Edit user</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconLock style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Change password
					</Menu.Item>

					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Delete this user
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
