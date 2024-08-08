"use client";

import Link from "next/link";
import React, { useState } from "react";

import styles from "./sidebar.module.scss";
import { XIconCpu, XIconMenu, XIconUpload } from "../icons";

const SideBar = () => {
	const [showCurrent, setShowCurrent] = useState(true);

	const toggleCurrent = () => {
		setShowCurrent(!showCurrent);
	};

	const links = [
		{
			name: "Main page",
			icon: <XIconCpu />,
			link: "/",
		},
		{
			name: "Upload files",
			icon: <XIconUpload />,
			link: "/upload",
		},
	];

	return (
		<div
			className={
				showCurrent
					? `${styles.sidebar}`
					: `${styles.sidebar} ${styles.sidebarhide}`
			}
		>
			<div className={styles.item} onClick={toggleCurrent}>
				<Link href="/">
					<div
						className={
							showCurrent
								? `${styles.itemicon} ${styles.hidebutton}`
								: `${styles.itemicon} ${styles.hidebutton} ${styles.hidebuttonhide}`
						}
					>
						{/* TODO: use mantine & place elsewhere */}
						<XIconMenu />
					</div>
				</Link>
			</div>

			<hr className={styles.break}></hr>

			{links.map(({ name, icon, link }, idx) => {
				return (
					<div key={idx} className={styles.item}>
						<Link href={link}>
							<div className={styles.itemicon}>{icon}</div>
							<div
								className={
									showCurrent
										? `${styles.itemtext}`
										: `${styles.itemtext} ${styles.itemtexthide}`
								}
							>
								{name}
							</div>
						</Link>
					</div>
				);
			})}
		</div>
	);
};

export default SideBar;
