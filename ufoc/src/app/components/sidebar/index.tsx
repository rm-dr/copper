"use client";

import Link from "next/link";
import React, { useState } from "react";

import styles from "./sidebar.module.scss";
import { useDisclosure } from "@mantine/hooks";
import { Burger, Tooltip } from "@mantine/core";
import clsx from "clsx";
import { usePathname } from "next/navigation";
import {
	IconDatabaseCog,
	IconListDetails,
	IconUpload,
	IconUsers,
} from "@tabler/icons-react";
import { XIcon } from "../icons";

const SideBar = () => {
	const [opened, { toggle }] = useDisclosure();
	const [menuHover, setMenuHover] = useState(false);
	const currentRoute = usePathname();

	const links = [
		{
			name: "Upload files",
			icon: <XIcon icon={IconUpload} />,
			link: "/u/upload",
		},
		{
			name: "Manage datasets",
			icon: <XIcon icon={IconDatabaseCog} />,
			link: "/u/datasets",
		},
		{
			name: "View items",
			icon: <XIcon icon={IconListDetails} />,
			link: "/u/items",
		},
		{
			name: "Manage users",
			icon: <XIcon icon={IconUsers} />,
			link: "/u/users",
		},
	];

	return (
		<div
			className={
				opened ? `${styles.sidebar}` : `${styles.sidebar} ${styles.sidebarhide}`
			}
		>
			<div
				className={styles.menubutton}
				onMouseDown={toggle}
				onMouseEnter={() => {
					setMenuHover(true);
				}}
				onMouseLeave={() => {
					setMenuHover(false);
				}}
			>
				<div className={styles.menuicon}>
					{/* Sizing here is broken, fix! */}
					<Burger
						opened={opened}
						//color={menuHover ? "var(--mantine-color-red-5)" : "white"}
					/>
				</div>
			</div>

			<hr className={styles.break}></hr>

			{links.map(({ name, icon, link }, idx) => {
				return (
					<Tooltip
						key={idx}
						label={name}
						position="right"
						offset={10}
						color="gray"
						disabled={opened}
					>
						<Link href={link}>
							<div
								className={clsx(
									styles.item,
									currentRoute == link && styles.itemactive,
								)}
							>
								<div className={styles.itemicon}>{icon}</div>
								<div
									className={clsx(
										styles.itemtext,
										!opened && styles.itemtexthide,
									)}
								>
									{name}
								</div>
							</div>
						</Link>
					</Tooltip>
				);
			})}
		</div>
	);
};

export default SideBar;
