"use client";

import styles from "./navbar.module.scss";

import Banner from "../../../../public/banner.svg";
import { Menu, Text, rem } from "@mantine/core";
import { XIcon } from "../icons";
import {
	IconBook2,
	IconLogout,
	IconSettings,
	IconUser,
} from "@tabler/icons-react";
import Link from "next/link";

const Navbar = () => {
	return (
		<div className={styles.navbar}>
			<div className={styles.banner}>
				<Banner />
			</div>

			<div className={styles.usermenu}>
				<Menu shadow="md">
					<Menu.Target>
						<Text>User</Text>
					</Menu.Target>

					<Menu.Dropdown>
						<Menu.Item
							leftSection={
								<XIcon
									icon={IconUser}
									style={{ width: rem(16), height: rem(16) }}
								/>
							}
						>
							Profile
						</Menu.Item>
						<Menu.Item
							leftSection={
								<XIcon
									icon={IconSettings}
									style={{ width: rem(16), height: rem(16) }}
								/>
							}
						>
							Settings
						</Menu.Item>
						<Menu.Item
							leftSection={
								<XIcon
									icon={IconLogout}
									style={{ width: rem(16), height: rem(16) }}
								/>
							}
							onClick={() => {
								fetch("/api/auth/logout", { method: "POST" }).then(() => {
									window.location.replace("/login");
								});
							}}
						>
							Log out
						</Menu.Item>

						<Menu.Divider />

						<Menu.Item
							leftSection={
								<XIcon
									icon={IconBook2}
									style={{ width: rem(16), height: rem(16) }}
								/>
							}
						>
							Documentation
						</Menu.Item>
					</Menu.Dropdown>
				</Menu>
			</div>
		</div>
	);
};

export default Navbar;
