import { ReactNode } from "react";
import styles from "./panel.module.scss";
import clsx from "clsx";

export const PanelTitle = (params: {
	icon: ReactNode;
	title: string;
	zeromargin?: boolean;
}) => {
	return (
		<div
			className={clsx(
				styles.panel_title,
				params.zeromargin === true && styles.panel_title_zeromargin,
			)}
		>
			<div className={styles.panel_icon}>{params.icon}</div>
			<div className={styles.panel_title_text}>{params.title}</div>
		</div>
	);
};

// A control panel. Each of these has a unique id,
// which is used to position and size the panel.
export const Panel = (params: {
	icon: any;
	title: string;
	children: any;
	panel_id: string;
}) => {
	return (
		<div className={styles.panel} id={params.panel_id}>
			<PanelTitle icon={params.icon} title={params.title} />
			<div className={styles.panel_content}>{params.children}</div>
		</div>
	);
};
