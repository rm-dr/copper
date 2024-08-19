import styles from "./users.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import { ActionIcon, Button, Menu, Text, rem } from "@mantine/core";
import { TreeNode } from "@/app/components/tree";
import { ReactNode, useState } from "react";
import { useAddUserModal } from "../_modals/adduser";
import { useDeleteUserModal } from "../_modals/deluser";
import { XIcon } from "@/app/components/icons";
import {
	IconCircleOff,
	IconDots,
	IconEdit,
	IconList,
	IconLock,
	IconTrash,
	IconUser,
	IconUserOff,
	IconUserPlus,
	IconUsers,
	IconUsersGroup,
} from "@tabler/icons-react";
import { components } from "@/app/_util/api/openapi";
import { useRenameUserModal } from "../_modals/renameuser";
import { useSetPasswordModal } from "../_modals/setpassword";
import { useUserInfoStore } from "@/app/_util/userinfo";

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
	data: TreeNode<components["schemas"]["ListgroupInfo"]>[];
	selected: string | null;
	onChange: () => void;
}) {
	// No loading state needed here,
	// since we re-use the data that our grouptree component fetches

	// g is `null` when no group is selected.
	// it should never be `undefined`.
	let g =
		params.selected === null
			? null
			: params.data.find((x) => x.uid === params.selected);

	const user_info = useUserInfoStore((state) => state.user_info);
	const { open: openModal, modal: addUserModal } = useAddUserModal({
		group: g?.data.group_info,
		onChange: params.onChange,
	});

	// This should never happen
	if (g === undefined) {
		return null;
	}

	let userlist = null;
	if (g === null) {
		userlist = (
			<Wrapper>
				<XIcon
					icon={IconCircleOff}
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
				<XIcon
					icon={IconUserOff}
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
			if (x.id === user_info?.id) {
				return (
					<div key={`${x.id}`} className={styles.user_entry}>
						<div className={styles.user_entry_icon}>
							<XIcon icon={IconUser} />
						</div>
						<div className={styles.user_entry_text}>
							{x.name}{" "}
							<Text c="dimmed" fs="italic" span>
								(you)
							</Text>
						</div>
						<div className={styles.user_entry_right}>
							<UserMenu user={x} onChange={params.onChange} />
						</div>
					</div>
				);
			} else {
				return (
					<div key={`${x.id}`} className={styles.user_entry}>
						<div className={styles.user_entry_icon}>
							<XIcon icon={IconUser} />
						</div>
						<div className={styles.user_entry_text}>{x.name}</div>
						<div className={styles.user_entry_right}>
							<UserMenu user={x} onChange={params.onChange} />
						</div>
					</div>
				);
			}
		});
	}

	return (
		<>
			{addUserModal}
			<Panel
				panel_id={styles.panel_users}
				icon={<XIcon icon={IconUsers} />}
				title={"Edit group"}
			>
				<PanelTitle icon={<XIcon icon={IconUsersGroup} />} title={"Overview"} />
				<div className={styles.overview_container}>
					<div className={styles.overview_entry}>
						<div className={styles.overview_entry_label}>Group:</div>
						<div className={styles.overview_entry_text}>
							{g === null ? (
								<Text c="dimmed">No group selected</Text>
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

				{/*
				TODO: implement when we do permissions
				<PanelTitle
					icon={<XIcon icon={IconSettings2} />}
					title={"Permissions"}
				/>
				<div className={styles.perm_container}>
					<div className={styles.perm_entry}>
						<div className={styles.perm_entry_switch}>
							<Switch defaultChecked size="xs" />
						</div>
						<div className={styles.perm_entry_text}>Edit datasets</div>
					</div>
					<div className={styles.perm_entry}>
						<div className={styles.perm_entry_switch}>
							<Switch defaultChecked size="xs" />
						</div>
						<div className={styles.perm_entry_text}>Edit groups</div>
					</div>
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
						<div className={styles.perm_entry_text}>
							Edit users parent group
						</div>
					</div>
				</div>
				*/}

				<PanelTitle icon={<XIcon icon={IconList} />} title={"Manage users"} />
				<Button
					radius="0"
					onClick={openModal}
					variant="light"
					color="green"
					size="xs"
					fullWidth
					disabled={g === null}
					leftSection={<XIcon icon={IconUserPlus} />}
					style={{ cursor: "default" }}
				>
					Create a user
				</Button>

				<div className={styles.users_container}>{userlist}</div>
			</Panel>
		</>
	);
}

function UserMenu(params: {
	user: components["schemas"]["UserInfo"];
	onChange: () => void;
}) {
	const { open: openDeleteModal, modal: deleteModal } = useDeleteUserModal({
		user: params.user,
		onChange: params.onChange,
	});

	const { open: openRenameModal, modal: renameModal } = useRenameUserModal({
		user: params.user,
		onChange: params.onChange,
	});

	const { open: openSetPasswordModal, modal: setPasswordModal } =
		useSetPasswordModal({
			user: params.user,
			onChange: params.onChange,
		});

	return (
		<>
			{deleteModal}
			{renameModal}
			{setPasswordModal}
			<Menu
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
				trigger="click-hover"
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIcon icon={IconDots} style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Edit user</Menu.Label>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconEdit}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openRenameModal}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIcon
								icon={IconLock}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openSetPasswordModal}
					>
						Change password
					</Menu.Item>

					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIcon
								icon={IconTrash}
								style={{ width: rem(14), height: rem(14) }}
							/>
						}
						onClick={openDeleteModal}
					>
						Delete this user
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}
