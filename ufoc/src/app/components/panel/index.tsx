import styles from "./panel.module.scss";

export const PanelTitle = ({ icon, title }: { icon: any; title: string }) => {
	return (
		<div className={styles.panel_title}>
			<div className={styles.panel_icon}>{icon}</div>
			<div className={styles.panel_title_text}>{title}</div>
		</div>
	);
};

// A control panel. Each of these has a unique id,
// which is used to position and size the panel.
export const Panel = ({
	icon,
	title,
	children,
	panel_id,
}: {
	icon: any;
	title: string;
	children: any;
	panel_id: string;
}) => {
	return (
		<div id={panel_id} className={styles.panel_container}>
			<div className={styles.panel}>
				<PanelTitle icon={icon} title={title} />
				{children}
			</div>
		</div>
	);
};

// A subsection inside a panel
export const PanelSection = ({
	icon,
	title,
	children,
}: {
	icon: any;
	title: string;
	children: any;
}) => {
	return (
		<div>
			<PanelTitle icon={icon} title={title} />
			<div className={styles.panel_content}>{children}</div>
		</div>
	);
};
