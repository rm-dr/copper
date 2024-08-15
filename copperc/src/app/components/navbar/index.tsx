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
	IconUserCancel,
	IconUserCircle,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";

const Navbar = () => {
	const [userInfo, setUserInfo] = useState<
		components["schemas"]["UserInfo"] | null | string
	>(null);

	useEffect(() => {
		APIclient.GET("/auth/me")
			.then(({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}
				setUserInfo(data);
			})
			.catch((e) => {
				setUserInfo("error");
			});
	}, []);

	return (
		<div className={styles.navbar}>
			<div className={styles.banner}>
				<Banner />
			</div>

			<div className={styles.usermenu}>
				<Menu trigger="click-hover" shadow="md">
					<Menu.Target>
						{typeof userInfo === "string" ? (
							<Text c="red">{userInfo}</Text>
						) : userInfo === null ? (
							<div className={styles.usercontainer}>
								<XIcon icon={IconUserCircle} />
								<Text c="dimmed">Loading...</Text>
							</div>
						) : (
							<div className={styles.usercontainer}>
								<XIcon icon={IconUserCircle} />
								<Text>{userInfo.name}</Text>
							</div>
						)}
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
