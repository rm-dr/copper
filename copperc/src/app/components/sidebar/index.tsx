"use client";

import Link from "next/link";
import React, { ReactElement } from "react";
import styles from "./sidebar.module.scss";
import { Tooltip } from "@mantine/core";
import clsx from "clsx";
import { usePathname } from "next/navigation";
import {
	IconDatabaseCog,
	IconListDetails,
	IconUpload,
	IconUserEdit,
	IconUsers,
} from "@tabler/icons-react";
import { XIcon } from "../icons";

const SideBar = () => {
	const currentRoute = usePathname();

	const links: (
		| {
				item: "link";
				name: string;
				icon: ReactElement;
				link: string;
		  }
		| { item: "break" }
	)[] = [
		{
			item: "link",
			name: "Manage profile",
			icon: <XIcon icon={IconUserEdit} />,
			link: "/u/profile",
		},
		{
			item: "link",
			name: "Manage users",
			icon: <XIcon icon={IconUsers} />,
			link: "/u/users",
		},

		{ item: "break" },

		{
			item: "link",
			name: "Manage datasets",
			icon: <XIcon icon={IconDatabaseCog} />,
			link: "/u/datasets",
		},
		{
			item: "link",
			name: "Upload files",
			icon: <XIcon icon={IconUpload} />,
			link: "/u/upload",
		},
		{
			item: "link",
			name: "View items",
			icon: <XIcon icon={IconListDetails} />,
			link: "/u/items",
		},
	];

	return (
		<div className={styles.sidebar}>
			{links.map((i, idx) => {
				if (i.item === "link") {
					return (
						<Tooltip
							key={idx}
							label={i.name}
							position="right"
							offset={10}
							color="gray"
						>
							<Link href={i.link}>
								<div
									className={clsx(
										styles.item,
										currentRoute == i.link && styles.itemactive,
									)}
								>
									<div className={styles.itemicon}>{i.icon}</div>
								</div>
							</Link>
						</Tooltip>
					);
				} else if (i.item === "break") {
					return <div key={idx} className={styles.break} />;
				}
			})}
		</div>
	);
};

export default SideBar;
