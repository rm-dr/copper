import SideBar from "./_sidebar/sidebar";
import TopBar from "./_topbar/topbar";
import styles from "./layout.module.scss";

export default function Layout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<div className={styles.top_bar_container}>{<TopBar />}</div>
			<div className={styles.lower_content}>
				<div className={styles.side_bar_container}>{<SideBar />}</div>
				<div className={styles.content_container}>{children}</div>
			</div>
		</>
	);
}
