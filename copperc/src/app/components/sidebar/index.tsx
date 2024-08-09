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
		<div className={styles.sidebar}>
			{links.map(({ name, icon, link }, idx) => {
				return (
					<Tooltip
						key={idx}
						label={name}
						position="right"
						offset={10}
						color="gray"
					>
						<Link href={link}>
							<div
								className={clsx(
									styles.item,
									currentRoute == link && styles.itemactive,
								)}
							>
								<div className={styles.itemicon}>{icon}</div>
							</div>
						</Link>
					</Tooltip>
				);
			})}
		</div>
	);
};

export default SideBar;
