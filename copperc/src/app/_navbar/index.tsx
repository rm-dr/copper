"use client";

import styles from "./navbar.module.scss";
import Banner from "../../../public/banner.svg";
import { Menu, Text, rem } from "@mantine/core";
import { XIcon } from "../../components/icons";
import {
	IconBook2,
	IconLogout,
	IconUser,
	IconUserCircle,
} from "@tabler/icons-react";
import { APIclient } from "@/lib/api";
import Link from "next/link";
import { useUserInfoStore } from "@/lib/userinfo";

const Navbar = () => {
	const user_info = useUserInfoStore((state) => state.user_info);

	return (
		<div className={styles.navbar}>
			<div className={styles.banner}>
				<Banner />
			</div>

			<div className={styles.usermenu}>
				<Menu trigger="click-hover" shadow="md">
					<Menu.Target>
						{user_info === null ? (
							<div className={styles.usercontainer}>
								<XIcon icon={IconUserCircle} />
								<Text c="dimmed">Loading...</Text>
							</div>
						) : (
							<div className={styles.usercontainer}>
								<XIcon icon={IconUserCircle} />
								<Text>{user_info.name}</Text>
							</div>
						)}
					</Menu.Target>

					<Menu.Dropdown>
						<Menu.Item
							component={Link}
							href={"/u/profile"}
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
									icon={IconLogout}
									style={{ width: rem(16), height: rem(16) }}
								/>
							}
							onClick={() => {
								APIclient.POST("/auth/logout").then(() => {
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
