"use client";

import { ReactElement } from "react";
import styles from "./sidebar.module.scss";
import { Tooltip } from "@mantine/core";
import clsx from "clsx";
import { usePathname } from "next/navigation";
import { Database, FileIcon, FileUp, House, Waypoints } from "lucide-react";
import { BlockableLink } from "@/components/navblock";

export function SideBar() {
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
			name: "Dashboard",
			icon: <House />,
			link: "/u/dashboard",
		},

		{
			item: "link",
			name: "Manage pipelines",
			icon: <Waypoints />,

			link: "/u/pipeline",
		},

		{
			item: "link",
			name: "Upload files",
			icon: <FileUp />,

			link: "/u/upload",
		},

		{ item: "break" },

		/*
		{
			item: "link",
			name: "Manage jobs",
			icon: <Logs />,
			link: "/u/jobs",
		},
		*/

		{
			item: "link",
			name: "Manage datasets",
			icon: <Database />,
			link: "/u/datasets",
		},

		{
			item: "link",
			name: "Manage items",
			icon: <FileIcon />,
			link: "/u/items",
		},
	];

	return (
		<div className={styles.sidebar}>
			{links.map((i, idx) => {
				if (i.item === "link") {
					const href = i.link;
					const inner = (
						<div
							className={clsx(
								styles.item,
								currentRoute == i.link && styles.itemactive,
							)}
						>
							<div className={styles.itemicon}>{i.icon}</div>
						</div>
					);

					return (
						<Tooltip
							key={idx}
							label={i.name}
							position="right"
							offset={10}
							color="gray"
						>
							{/* Div is necessary because function components cannot be given refs */}
							<div>
								<BlockableLink href={href}>{inner}</BlockableLink>
							</div>
						</Tooltip>
					);
				} else if (i.item === "break") {
					return <div key={idx} className={styles.break} />;
				}
			})}
		</div>
	);
}

export default SideBar;
